    import { gen_index } from './general-helper';
    import { get_cookie, has_cookie, set_cookie } from "./cookies";
    import { auth_fetch, get_user_info } from './auth';

    // SSE event messages arrive as "<topic>:::<payload>". These topics must match
    // the header defines the backend prepends (see defines: KEY_CONTROLS_EVENT).
    const SSE_TOPIC_DELIMITER = ':::';
    const SSE_TOPIC_CONTROLS_EVENT = 'CONTROLS_EVENT'; // log line payload
    const SSE_TOPIC_TASK_EVENT = 'TASK_EVENT';         // task queue changed

    // One entry in the beamline selector dropdown. `acronym` doubles as the
    // page's beamline_id; `name` is the human-readable label.
    interface BeamlineInfo
    {
        acronym: string;
        name: string;
    }

    interface BeamlineLog
    {
        time: number;
        msg: string;
    }

    type BeamlineLogs = Array<BeamlineLog>;


    interface ParameterKind 
    {
        name: string;
        value: number;
    }

    interface Parameter 
    {
        description: string;
        kind: ParameterKind;
        name: string;
        default?: string; // Optional property
    }

    interface PlanProperties 
    {
        is_generator: boolean;
    }

    interface Plan 
    {
        description: string;
        module: string;
        name: string;
        parameters: Parameter[];
        properties: PlanProperties;
    }

    interface PlansAllowed 
    {
        [key: string] : Plan; // Allows for dynamic keys
    }

    interface ScanApiResponse 
    {
        success: boolean;
        msg: string;
        plans_allowed: PlansAllowed;
    }

    interface CommandArgs
    {
         [key: string]: string
    }
    interface BeamlineTask
    {
        beamline_id: string,
        status: string,
        cmd: string,
        args: CommandArgs,
        username?: string,
        reply?: string,
        proc_start_time?: string, // DateTime<Utc>,
        proc_end_time?: string, //DateTime<Utc>,
        original_str?: string,
    }
    interface BeamlineTasks
    {
        beamline_id: string,
        queued: Array<BeamlineTask>,
        processing: Array<BeamlineTask>,
        done: Array<BeamlineTask>,
    }

    class ScanPlan
    {


    }

    class BeamlineLogWidget
    {

        private beamline_id: string;
        private logs: BeamlineLogs | null;
        private tasks: BeamlineTask | null;

        private main_div: HTMLDivElement;
        private tab_div: HTMLDivElement;
        private content_div: HTMLDivElement;
        private logs_div: HTMLDivElement;
        private tasks_div: HTMLDivElement;
        private tab_link_logs: HTMLButtonElement;
        private tab_link_tasks: HTMLButtonElement;
        private beamline_event_source: EventSource;
        // Which panel is expanded ('logs' | 'tasks'), or null when the dock is collapsed.
        private active_tab: string | null;

        constructor(beam_id: string)
        {
            this.main_div = document.createElement("div") as HTMLDivElement;
            this.main_div.id = "beamline-log-main";
            this.main_div.classList.add("beamline-log-main");
            this.main_div.classList.add("collapsed");
            this.tab_div = document.createElement("div") as HTMLDivElement;
            this.tab_div.id = "beamline-tab";
            this.tab_div.classList.add("tab");

            // Scrollable content area that expands above the tab bar. Holds both
            // panels; only the active one is shown.
            this.content_div = document.createElement("div") as HTMLDivElement;
            this.content_div.id = "beamline-log-content";
            this.content_div.classList.add("beamline-log-content");
            this.content_div.classList.add("hidden");

            this.logs_div = document.createElement("div") as HTMLDivElement;
            this.logs_div.id = "beamline-logs";
            this.logs_div.classList.add("beamline-log-panel");
            this.tasks_div = document.createElement("div") as HTMLDivElement;
            this.tasks_div.id = "beamline-tasks";
            this.tasks_div.classList.add("beamline-log-panel");
            this.tasks_div.classList.add("hidden");
            this.logs = null;
            this.tasks = null;
            this.active_tab = null;
            this.beamline_id = beam_id;

            this.tab_link_logs = document.createElement('button') as HTMLButtonElement;
            this.tab_link_logs.innerText = "Logs";
            this.tab_link_logs.classList.add('beamline-log-tab');
            this.tab_link_logs.addEventListener('click', (event)=> { this.toggleTab('logs'); });
            this.tab_link_tasks = document.createElement('button') as HTMLButtonElement;
            this.tab_link_tasks.innerText = "Tasks";
            this.tab_link_tasks.classList.add('beamline-log-tab');
            this.tab_link_tasks.addEventListener('click', (event)=> { this.toggleTab('tasks'); });


            this.tab_div.appendChild(this.tab_link_logs);
            this.tab_div.appendChild(this.tab_link_tasks);

            this.content_div.appendChild(this.logs_div);
            this.content_div.appendChild(this.tasks_div);

            // Tab bar sits below the content so the dock grows upward when expanded.
            this.main_div.appendChild(this.content_div);
            this.main_div.appendChild(this.tab_div);

            let sse_url = "api/sse/"+beam_id;
            this.beamline_event_source = new EventSource(sse_url, { withCredentials: true} );
            this.beamline_event_source.onopen = (event) => 
            {
                console.log('SSE connection established.', event);
            };
            
            this.beamline_event_source.onmessage = (event) =>
            {
                console.log("Received SSE message:", event.data);
                this.handleSseMessage(event.data);
            };

            this.beamline_event_source.onerror = (error) => 
            {
                console.error("SSE error:", error);
                this.beamline_event_source.close();
            };
            
            this.fetcLogs().then(logs => 
            {
                //this.logs = logs
                this.populateLogs(logs);
            }
            );

            this.fetcTasks().then(Tasks => 
            {
                //this.tasks = Tasks
                this.populateTasks(Tasks);
            }
            );
        }

        public gen_main_div(): HTMLDivElement
        {
            return this.main_div;
        }

        // Dispatch an SSE event based on its "<topic>:::<payload>" header:
        // log events are appended to the logs; task events refresh the task queues.
        private handleSseMessage(data: string): void
        {
            const delim_idx = data.indexOf(SSE_TOPIC_DELIMITER);
            if (delim_idx === -1)
            {
                console.warn("SSE message missing topic delimiter:", data);
                return;
            }
            const topic = data.substring(0, delim_idx);
            const payload = data.substring(delim_idx + SSE_TOPIC_DELIMITER.length);

            switch (topic)
            {
                case SSE_TOPIC_CONTROLS_EVENT:
                {
                    try
                    {
                        const blog = JSON.parse(payload) as BeamlineLog;
                        this.appendLog(blog);
                    }
                    catch (error)
                    {
                        console.error("Failed to parse log event payload:", error, payload);
                    }
                    break;
                }
                case SSE_TOPIC_TASK_EVENT:
                {
                    // A task changed; re-fetch the queues and repopulate.
                    this.fetcTasks().then(tasks => this.populateTasks(tasks));
                    break;
                }
                default:
                    console.warn("Unknown SSE topic:", topic);
            }
        }

        // Clicking a tab expands the dock (up to a third of the screen) to show
        // that panel; clicking the same tab again collapses the dock.
        private toggleTab(tab: string): void
        {
            this.active_tab = (this.active_tab === tab) ? null : tab;
            this.applyTabState();
        }

        // Reflect `active_tab` in the DOM: expand/collapse the dock, show the
        // selected panel, and highlight the active tab button.
        private applyTabState(): void
        {
            const expanded = this.active_tab !== null;
            this.main_div.classList.toggle('expanded', expanded);
            this.main_div.classList.toggle('collapsed', !expanded);
            this.content_div.classList.toggle('hidden', !expanded);

            const logs_active = this.active_tab === 'logs';
            const tasks_active = this.active_tab === 'tasks';
            this.logs_div.classList.toggle('hidden', !logs_active);
            this.tasks_div.classList.toggle('hidden', !tasks_active);
            this.tab_link_logs.classList.toggle('active', logs_active);
            this.tab_link_tasks.classList.toggle('active', tasks_active);
        }

        private async fetcLogs(): Promise<BeamlineLogs | null>
        {
            try 
            {
                const response = await fetch('/api/get_beamline_log/'+this.beamline_id);

                if (!response.ok) 
                {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                
                //this.logs = await response.json();
                const json_repl: BeamlineLogs = await response.json();
                return json_repl;
            }
            catch (error) 
            {
                console.error('Error loading sample types:', error);
                return null;
                //this.showMessage('Failed to load sample types. Using default values.', 'error');
            }
        }

        private async fetcTasks(): Promise<BeamlineTasks | null>
        {
            try 
            {
                const response = await fetch('/api/get_beamline_worker_task_queues/'+this.beamline_id);

                if (!response.ok) 
                {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                
                //this.logs = await response.json();
                const json_repl: BeamlineTasks = await response.json();
                return json_repl;
            }
            catch (error) 
            {
                console.error('Error loading sample types:', error);
                return null;
                //this.showMessage('Failed to load sample types. Using default values.', 'error');
            }
        }

        private populateLogs(nlogs: BeamlineLogs | null): void
        {
            //console.log(nlogs);
            this.logs_div.innerText = "";

            //nlogs?.reverse();
            let htmlList = document.createElement("ul") as HTMLUListElement;
            nlogs?.forEach((item: BeamlineLog) => 
            {
                let list_item = document.createElement("li") as HTMLLIElement;
                list_item.textContent  = item.msg;
                htmlList.appendChild(list_item);
            });
            this.logs_div.appendChild(htmlList);
        }

        private appendLog(item: BeamlineLog | null): void
        {
            if(item)
            {
                let htmlList = document.createElement("ul") as HTMLUListElement;
                let list_item = document.createElement("li") as HTMLLIElement;
                list_item.textContent  = item.msg;
                htmlList.appendChild(list_item);
                this.logs_div.appendChild(htmlList);
            }
        }

        private populateTasks(bTasks: BeamlineTasks | null): void
        {
            //console.log(nlogs);
            this.tasks_div.innerText = "";

            //nlogs?.reverse();
            let table = document.createElement("table") as HTMLTableElement;
            let hrow = table.insertRow();
            let th0 = document.createElement("th");
            th0.innerText = "Status";
            hrow.appendChild(th0);
            let th1 = document.createElement("th");
            th1.innerText = "Command";
            hrow.appendChild(th1);
            let th2 = document.createElement("th");
            th2.innerText = "Username";
            hrow.appendChild(th2);
            let th3 = document.createElement("th");
            th3.innerText = "Reply";
            hrow.appendChild(th3);
            let th4 = document.createElement("th");
            th4.innerText = "Start Time";
            hrow.appendChild(th4);
            let th5 = document.createElement("th");
            th5.innerText = "End Time";
            hrow.appendChild(th5);
            bTasks?.queued.forEach((item: BeamlineTask) => 
            {
                let row = table.insertRow();
                let cell0 = row.insertCell();
                cell0.innerText = item.status;
                let cell1 = row.insertCell();
                cell1.innerText = item.cmd;
                let cell2 = row.insertCell();
                cell2.innerText = item.username!;
                let cell3 = row.insertCell();
                let div = document.createElement('div') as HTMLDivElement;
                div.innerText = item.reply!;
                div.classList.add('scrollable');
                cell3.appendChild(div);
                let cell4 = row.insertCell();
                cell4.innerText = item.proc_start_time!;
                let cell5 = row.insertCell();
                cell5.innerText = item.proc_end_time!;
                
            });
            bTasks?.processing.forEach((item: BeamlineTask) => 
            {
                let row = table.insertRow();
                let cell0 = row.insertCell();
                cell0.innerText = item.status;
                let cell1 = row.insertCell();
                cell1.innerText = item.cmd;
                let cell2 = row.insertCell();
                cell2.innerText = item.username!;
                let cell3 = row.insertCell();
                let div = document.createElement('div') as HTMLDivElement;
                div.innerText = item.reply!;
                div.classList.add('scrollable');
                cell3.appendChild(div);
                let cell4 = row.insertCell();
                cell4.innerText = item.proc_start_time!;
                let cell5 = row.insertCell();
                cell5.innerText = item.proc_end_time!;
                
            });
            bTasks?.done.forEach((item: BeamlineTask) => 
            {
                let row = table.insertRow();
                let cell0 = row.insertCell();
                cell0.innerText = item.status;
                let cell1 = row.insertCell();
                cell1.innerText = item.cmd;
                let cell2 = row.insertCell();
                cell2.innerText = item.username!;
                let cell3 = row.insertCell();
                let div = document.createElement('div') as HTMLDivElement;
                div.innerText = item.reply!;
                div.classList.add('scrollable');
                cell3.appendChild(div);
                let cell4 = row.insertCell();
                cell4.innerText = item.proc_start_time!;
                let cell5 = row.insertCell();
                cell5.innerText = item.proc_end_time!;
                
            });
            this.tasks_div.appendChild(table);
        }
    }
 
    class BeamlineScansWidget
    {
        private beamline_id: string;
        private main_div: HTMLDivElement;
        private queued_div: HTMLDivElement;
        private processing_div: HTMLDivElement;
        private done_div: HTMLDivElement;
        
        private available_scans: ScanApiResponse | null;

        constructor(beam_id: string)
        {
            this.main_div = document.createElement("div") as HTMLDivElement;
            this.main_div.id = "beamline-scans";
            this.main_div.classList.add("beamline-scans");

            this.queued_div = document.createElement("div") as HTMLDivElement;
            this.queued_div.id = "beamline-scans-queued";
            this.queued_div.classList.add("beamline-scans");
            
            this.processing_div = document.createElement("div") as HTMLDivElement;
            this.processing_div.id = "beamline-scans-processing";
            this.processing_div.classList.add("beamline-scans");
            
            this.done_div = document.createElement("div") as HTMLDivElement;
            this.done_div.id = "beamline-scans-done";
            this.done_div.classList.add("beamline-scans");
            

            this.main_div.appendChild(this.queued_div);
            this.main_div.appendChild(this.processing_div);
            this.main_div.appendChild(this.done_div);

            this.available_scans = null;
            this.beamline_id = beam_id;

            this.fetchAvailableScans().then(scans =>
            {
                this.available_scans = scans;
                this.populateAvailableScans(scans);
            }
            );
        }

        public gen_main_div(): HTMLDivElement
        {
            return this.main_div;
        }

        private async fetchAvailableScans(): Promise<ScanApiResponse | null>
        {
            try 
            {
                const response = await fetch('/api/get_available_scans/'+this.beamline_id);

                if (!response.ok) 
                {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                
                //this.logs = await response.json();
                const json_repl: ScanApiResponse = await response.json();
                return json_repl;
            }
            catch (error) 
            {
                console.error('Error loading sample types:', error);
                //this.showMessage('Failed to load sample types. Using default values.', 'error');
                return null;
            }
        }
/*
        private async fetchScanQueue(): Promise<ScanApiResponse | null>
        {
            try 
            {
                const response = await fetch('/api/get_available_scans/'+this.beamline_id);

                if (!response.ok) 
                {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                
                //this.logs = await response.json();
                const json_repl: ScanApiResponse = await response.json();
                return json_repl;
            }
            catch (error) 
            {
                console.error('Error loading sample types:', error);
                //this.showMessage('Failed to load sample types. Using default values.', 'error');
                return null;
            }
        }
*/
        // Render the available scan plans parsed from the redis value into
        // viewable components: one card per plan showing its description and a
        // table of parameters.
        private populateAvailableScans(scans: ScanApiResponse | null): void
        {
            this.queued_div.innerText = "";

            // Section header with a toggle to collapse/expand the whole plan list.
            const header = document.createElement("div") as HTMLDivElement;
            header.classList.add("scan-section-header");

            const toggle = document.createElement("button") as HTMLButtonElement;
            toggle.type = "button";
            toggle.classList.add("scan-plan-toggle");
            toggle.innerText = "+";

            const heading = document.createElement("h3") as HTMLHeadingElement;
            heading.innerText = "Available Scan Plans";

            header.appendChild(toggle);
            header.appendChild(heading);
            this.queued_div.appendChild(header);

            // Container holding all plan cards; toggled as a unit. Starts collapsed.
            const list = document.createElement("div") as HTMLDivElement;
            list.classList.add("scan-plan-list");
            list.classList.add("hidden");
            this.queued_div.appendChild(list);

            const do_toggle = () =>
            {
                const collapsed = list.classList.toggle("hidden");
                toggle.innerText = collapsed ? "+" : "-";
            };
            toggle.addEventListener('click', do_toggle);
            heading.addEventListener('click', do_toggle);

            if (scans?.success !== true || !scans.plans_allowed)
            {
                const msg = document.createElement("div") as HTMLDivElement;
                msg.innerText = scans?.msg ? scans.msg : "No available scan plans.";
                list.appendChild(msg);
                return;
            }

            const plan_names = Object.keys(scans.plans_allowed).sort();
            plan_names.forEach(key =>
            {
                const plan = scans.plans_allowed[key];
                if (plan)
                {
                    list.appendChild(this.gen_plan_card(key, plan));
                }
            });
        }

        // Build a single scan plan card: a collapsed row showing only the name
        // with a '+'/'-' toggle button that expands to reveal the description
        // and parameters table.
        private gen_plan_card(name: string, plan: Plan): HTMLDivElement
        {
            const card = document.createElement("div") as HTMLDivElement;
            card.classList.add("scan-plan");

            const header = document.createElement("div") as HTMLDivElement;
            header.classList.add("scan-plan-header");

            const toggle = document.createElement("button") as HTMLButtonElement;
            toggle.type = "button";
            toggle.classList.add("scan-plan-toggle");
            toggle.innerText = "+";

            const title = document.createElement("span") as HTMLSpanElement;
            title.classList.add("scan-plan-name");
            title.innerText = plan.name ? plan.name : name;

            header.appendChild(toggle);
            header.appendChild(title);
            card.appendChild(header);

            const details = this.gen_plan_form(name, plan);
            details.classList.add("hidden");
            card.appendChild(details);

            const do_toggle = () =>
            {
                const collapsed = details.classList.toggle("hidden");
                toggle.innerText = collapsed ? "+" : "-";
            };
            // Clicking the toggle button or anywhere on the header row expands/collapses.
            toggle.addEventListener('click', do_toggle);
            title.addEventListener('click', do_toggle);

            return card;
        }

        // Build the expandable details section for a plan as a fillable form:
        // a description, one input per parameter (pre-filled with its default),
        // and a submit button that queues the plan with the entered values.
        private gen_plan_form(name: string, plan: Plan): HTMLFormElement
        {
            const form = document.createElement("form") as HTMLFormElement;
            form.classList.add("scan-plan-details");

            if (plan.description)
            {
                const desc = document.createElement("div") as HTMLDivElement;
                desc.classList.add("scan-plan-desc");
                desc.innerText = plan.description;
                form.appendChild(desc);
            }

            // Map each parameter name to its input so we can collect values on submit.
            const inputs: Map<string, HTMLInputElement> = new Map();

            if (plan.parameters && plan.parameters.length > 0)
            {
                const table = document.createElement("table") as HTMLTableElement;
                table.classList.add("scan-plan-params");
                const hrow = table.insertRow();
                ["Parameter", "Value", "Description"].forEach(text =>
                {
                    const th = document.createElement("th");
                    th.innerText = text;
                    hrow.appendChild(th);
                });

                plan.parameters.forEach((param: Parameter) =>
                {
                    const row = table.insertRow();
                    row.insertCell().innerText = param.name;

                    const value_cell = row.insertCell();
                    const input = document.createElement("input") as HTMLInputElement;
                    input.type = "text";
                    input.name = param.name;
                    input.classList.add("scan-plan-input");
                    if (param.default !== undefined)
                    {
                        input.value = param.default;
                        input.placeholder = param.default;
                    }
                    value_cell.appendChild(input);
                    inputs.set(param.name, input);

                    row.insertCell().innerText = param.description ? param.description : "";
                });

                form.appendChild(table);
            }

            const submit_btn = document.createElement("button") as HTMLButtonElement;
            submit_btn.type = "submit";
            submit_btn.classList.add("scan-plan-submit");
            submit_btn.innerText = "Queue Scan";
            form.appendChild(submit_btn);

            const plan_name = plan.name ? plan.name : name;
            form.addEventListener('submit', (event) =>
            {
                event.preventDefault();
                const args: CommandArgs = {};
                inputs.forEach((input, param_name) =>
                {
                    // Only send parameters the user actually provided a value for.
                    if (input.value !== "")
                    {
                        args[param_name] = input.value;
                    }
                });
                this.submit_plan(plan_name, args);
            });

            return form;
        }

        // Queue a scan plan on the beamline worker with the values entered in
        // the plan's form.
        private async submit_plan(plan_name: string, args: CommandArgs): Promise<void>
        {
            try
            {
                const auth_cookie: string = get_cookie('access_token');
                const headers = new Headers({
                    'Content-Type': 'application/json',
                    'Accept': 'application/json',
                    'Authorization': auth_cookie,
                });
                const command: BeamlineTask = {
                    status: " ",
                    beamline_id: this.beamline_id,
                    cmd: plan_name,
                    args: args,
                };
                console.log(command);
                const requestOptions: RequestInit = {
                    method: 'POST',
                    headers: headers,
                    body: JSON.stringify(command),
                };

                const response = await auth_fetch('/api/queue_beamline_worker_task/' + this.beamline_id, requestOptions);
                console.log(response);
            }
            catch (error)
            {
                console.error('Error queueing scan plan:', error);
            }
        }

    }

    class BeamlineWidget
    {
        private scans_widget: BeamlineScansWidget;
        private refresh_plans_btn: HTMLButtonElement;
        private logs_widget: BeamlineLogWidget;
        private main_div: HTMLDivElement;
        private beamline_id: string;

        constructor(beam_id: string)
        {
            this.main_div = document.createElement("div") as HTMLDivElement;
            this.main_div.id = "beamline-widget";
            this.main_div.classList.add("beamline-widget");
            this.scans_widget = new BeamlineScansWidget(beam_id);
            this.logs_widget = new BeamlineLogWidget(beam_id);
            this.beamline_id = beam_id;
            this.refresh_plans_btn = document.createElement("button") as HTMLButtonElement;
            this.refresh_plans_btn.innerText="Refresh Scan Plans";
            this.refresh_plans_btn.addEventListener('click', (event)=>
            { this.refresh_plans(); });

            this.main_div.appendChild(this.refresh_plans_btn);
            this.main_div.appendChild(this.scans_widget.gen_main_div())
            this.main_div.appendChild(this.logs_widget.gen_main_div())
            
        }
        
        public gen_main_div(): HTMLDivElement
        {
            return this.main_div;
        }

        private async refresh_plans(): Promise<void>
        {
            try 
            {
                const auth_cookie:string = get_cookie('access_token');

                const headers = new Headers({
                    'Content-Type': 'application/json',
                    'Accept': 'application/json',
                    'Authorization': auth_cookie,
                });
                let cargs: CommandArgs = {
                    user_group: 'primary'
                }
                let command: BeamlineTask = {
                    status: " ",
                    beamline_id: this.beamline_id,
                    cmd: "plans_allowed",
                    args: cargs,
                }
                console.log(command);
                const requestOptions: RequestInit = {
                    method: 'POST',
                    headers: headers,
                    body: JSON.stringify(command),
                };

                
                const response = await auth_fetch('/api/queue_beamline_worker_task/'+this.beamline_id, requestOptions);
                const data = await response;
                console.log(data);
            }
            catch (error) 
            {
                console.error('Error refreshing plans:', error);
            }
        }
    }

// Fetch every beamline for the selector dropdown.
async function fetch_all_beamlines(): Promise<Array<BeamlineInfo>>
{
    const auth_cookie: string = get_cookie('access_token');
    const headers = new Headers({
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'Authorization': auth_cookie,
    });
    const response = await auth_fetch('/api/get_all_beamlines', { method: 'GET', headers: headers });
    if (!response.ok)
    {
        throw new Error(`HTTP error! status: ${response.status}`);
    }
    return await response.json() as Array<BeamlineInfo>;
}

// Fetch the beamline(s) the logged-in user is a contact for. The backend joins
// beamlines, beamline_contacts, and users on the caller's badge.
async function fetch_my_beamlines(): Promise<Array<BeamlineInfo>>
{
    const auth_cookie: string = get_cookie('access_token');
    const headers = new Headers({
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'Authorization': auth_cookie,
    });
    const response = await auth_fetch('/api/get_my_beamlines', { method: 'GET', headers: headers });
    if (!response.ok)
    {
        throw new Error(`HTTP error! status: ${response.status}`);
    }
    return await response.json() as Array<BeamlineInfo>;
}

// Build the beamline selector. Selecting an entry reloads the page for the chosen
// beamline, passing its acronym as the beamline_id URL parameter.
async function gen_beamline_selector(current: string): Promise<HTMLDivElement>
{
    const wrap = document.createElement("div") as HTMLDivElement;
    wrap.classList.add("beamline-selector");

    const label = document.createElement("label") as HTMLLabelElement;
    label.textContent = "Beamline: ";
    label.htmlFor = "beamline-select";

    const select = document.createElement("select") as HTMLSelectElement;
    select.id = "beamline-select";

    try
    {
        const beamlines = await fetch_all_beamlines();
        beamlines.forEach(bl =>
        {
            const opt = document.createElement("option") as HTMLOptionElement;
            opt.value = bl.acronym;
            opt.textContent = bl.name ? `${bl.acronym} — ${bl.name}` : bl.acronym;
            if (bl.acronym === current)
            {
                opt.selected = true;
            }
            select.appendChild(opt);
        });
    }
    catch (error)
    {
        console.error('Error loading beamlines:', error);
    }

    select.addEventListener('change', () =>
    {
        window.location.href = 'beamline.html?beamline_id=' + encodeURIComponent(select.value);
    });

    wrap.appendChild(label);
    wrap.appendChild(select);
    return wrap;
}

// Initialize the application when the DOM is loaded
document.addEventListener('DOMContentLoaded', async () =>
{
    const queryString = window.location.search;
    const urlParams = new URLSearchParams(queryString);
    let beamline_id = urlParams.get('beamline_id')?.toString() ?? '';

    // With no beamline explicitly requested, staff default to their assigned
    // beamline (queried from beamlines/beamline_contacts/users).
    if (beamline_id.length === 0)
    {
        try
        {
            const claims = await get_user_info();
            if (claims.uac === 'Staff' || claims.uac === 'Admin')
            {
                const mine = await fetch_my_beamlines();
                if (mine.length > 0 && mine[0])
                {
                    beamline_id = mine[0].acronym;
                }
            }
        }
        catch (error)
        {
            console.error('Could not determine assigned beamline:', error);
        }
    }

    // Fall back to the historical default if nothing else resolved.
    if (beamline_id.length === 0)
    {
        beamline_id = 'sec0';
    }

    const container = document.createElement("div") as HTMLDivElement;
    container.appendChild(await gen_beamline_selector(beamline_id));

    const blw = new BeamlineWidget(beamline_id);
    container.appendChild(blw.gen_main_div());

    gen_index('app', container);
});