    import { gen_index } from './general-helper';
    import { get_cookie, has_cookie, set_cookie } from "./cookies";
    
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
        private logs_div: HTMLDivElement;
        private tasks_div: HTMLDivElement;
        private tab_link_logs: HTMLButtonElement;
        private tab_link_tasks: HTMLButtonElement;
        private beamline_event_source: EventSource;

        constructor(beam_id: string)
        {
            this.main_div = document.createElement("div") as HTMLDivElement;
            this.main_div.id = "beamline-log-main";
            this.main_div.classList.add("beamline-log-main");
            this.tab_div = document.createElement("div") as HTMLDivElement;
            this.tab_div.id = "beamline-tab";
            this.tab_div.classList.add("tab");
            
            this.logs_div = document.createElement("div") as HTMLDivElement;
            this.logs_div.id = "beamline-logs";
            this.logs_div.classList.add("beamline-log");
            this.tasks_div = document.createElement("div") as HTMLDivElement;
            this.tasks_div.id = "beamline-tasks";
            this.tasks_div.classList.add("beamline-log");
            this.tasks_div.classList.add("hidden");
            this.logs = null;
            this.tasks = null;
            this.beamline_id = beam_id; 

            this.tab_link_logs = document.createElement('button') as HTMLButtonElement;
            this.tab_link_logs.innerText = "Logs";
            this.tab_link_logs.classList.add('beamline-log-tab');
            this.tab_link_logs.addEventListener('click', (event)=> { this.setlogsVisible(); });
            this.tab_link_tasks = document.createElement('button') as HTMLButtonElement;
            this.tab_link_tasks.innerText = "Tasks";
            this.tab_link_tasks.classList.add('beamline-log-tab');
            this.tab_link_tasks.addEventListener('click', (event)=> { this.setTasksVisible(); });


            this.tab_div.appendChild(this.tab_link_logs);
            this.tab_div.appendChild(this.tab_link_tasks);

            this.main_div.appendChild(this.tab_div);
            this.main_div.appendChild(this.logs_div);
            this.main_div.appendChild(this.tasks_div);

            let sse_url = "api/sse/"+beam_id;
            this.beamline_event_source = new EventSource(sse_url, { withCredentials: true} );
            this.beamline_event_source.onopen = (event) => 
            {
                console.log('SSE connection established.', event);
            };
            
            this.beamline_event_source.onmessage = (event) => 
            {
                console.log("Received SSE message:", event.data);
                // append to logs
                const blog: BeamlineLog = event.data as BeamlineLog;
                this.appendLog(blog);
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

        private setlogsVisible(): void
        {
            this.logs_div.classList.remove('hidden')
            this.tasks_div.classList.add('hidden')
        }

        private setTasksVisible(): void
        {
            this.logs_div.classList.add('hidden')
            this.tasks_div.classList.remove('hidden')
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
                //this.populateAvailableScans(scans);
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
        private populateAvailableScans(scans: ScanApiResponse | null): void
        {
            console.log(scans);
            this.queued_div.innerText = "";

            if (scans?.success === true)
            {
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

                let htmlList = document.createElement("ul") as HTMLUListElement;
                Object.keys(scans.plans_allowed).forEach(key => 
                {
                    //console.log(key, scans.plans_allowed[key]);
                    let list_item = document.createElement("li") as HTMLLIElement;
                    list_item.textContent  = key; //scans.plans_allowed[key].name;
                    htmlList.appendChild(list_item);
                });
                
                this.main_div.appendChild(htmlList);
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

                
                const response = await fetch('/api/queue_beamline_worker_task/'+this.beamline_id, requestOptions);
                const data = await response;
                console.log(data);
            }
            catch (error) 
            {
                console.error('Error refreshing plans:', error);
            }
        }
    }

// Initialize the application when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => 
{
    const queryString = window.location.search;
    const urlParams = new URLSearchParams(queryString);
    let beamline_id = urlParams.get('beamline_id')?.toString() ?? 'sec0';
    let blw = new BeamlineWidget(beamline_id);
    gen_index('app', blw.gen_main_div());
});