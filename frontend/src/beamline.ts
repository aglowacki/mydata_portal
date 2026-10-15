    import { gen_index } from './general-helper';
    import { get_cookie, has_cookie, set_cookie } from "./cookies";
    import { auth_fetch, get_user_info } from './auth';

    // SSE event messages arrive as "<topic>:::<payload>". These topics must match
    // the header defines the backend prepends (see defines: KEY_CONTROLS_EVENT).
    const SSE_TOPIC_DELIMITER = ':::';
    const SSE_TOPIC_CONTROLS_EVENT = 'CONTROLS_EVENT'; // log line payload
    const SSE_TOPIC_TASK_EVENT = 'TASK_EVENT';         // task queue changed
    const SSE_TOPIC_QUEUE_EVENT = 'QUEUE_EVENT';       // scan (plan) queue changed
    const SSE_TOPIC_HISTORY_EVENT = 'HISTORY_EVENT';   // scan finished; history changed

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

    // One entry in the queueserver plan queue (or the running item). Mirrors the
    // subset of fields returned by the queueserver `queue_get` API we display.
    interface QueueItem
    {
        item_type?: string;   // "plan" | "instruction"
        name?: string;
        args?: unknown[];
        kwargs?: { [key: string]: unknown };
        item_uid?: string;
        user?: string;
        user_group?: string;
    }

    // Response shape of the queueserver `queue_get` API, cached by the worker and
    // served verbatim by /api/get_queued_scans/<beamline_id>.
    interface QueueGetResponse
    {
        success: boolean;
        msg: string;
        items: QueueItem[];
        running_item: QueueItem | Record<string, never>;
        plan_queue_uid: string;
    }

    // The per-item `result` block the queueserver attaches to finished scans in the
    // history. Only the fields we display/inspect are declared.
    interface HistoryResult
    {
        exit_status?: string;      // "completed" | "failed" | "aborted" | "halted" | ...
        time_start?: number;       // unix seconds
        time_stop?: number;        // unix seconds
        msg?: string;
        traceback?: string;
    }

    // One finished scan in the queueserver history: a QueueItem plus its result.
    interface HistoryItem extends QueueItem
    {
        result?: HistoryResult;
    }

    // Response shape of the queueserver `history_get` API, cached by the worker and
    // served verbatim by /api/get_scan_history/<beamline_id>.
    interface HistoryGetResponse
    {
        success: boolean;
        msg: string;
        items: HistoryItem[];
        plan_history_uid: string;
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

    // One element's 2-D XRF map returned by /api/get_xrf_streaming_cache. `data` is
    // row-major (`height` rows of `width` values); never-written pixels arrive as
    // null. `datasets`/`elements` list what else is available to switch to.
    interface XrfMap
    {
        dataset: string;
        element: string;
        width: number;
        height: number;
        datasets: string[];
        elements: string[];
        data: (number | null)[];
    }

    // --- Scheduler activity shapes (minimal subset of the upstream API) --------
    // Mirrors the shapes used on the scheduler page; only the fields shown in the
    // "current activity" panel are declared.
    interface SchedExperimenter
    {
        firstName?: string;
        lastName?: string;
        institution?: string;
        piFlag?: string | null;
    }

    interface SchedProposal
    {
        gupId?: number | null;
        proposalTitle?: string | null;
        experimenters?: Array<SchedExperimenter>;
    }

    interface SchedActivity
    {
        activityName?: string | null;
        startTime?: string | null;
        endTime?: string | null;
        beamtime?: { proposal?: SchedProposal };
        station?: { stationName?: string | null } | null;
        user?: { name?: string | null; firstName?: string | null; lastName?: string | null; userName?: string | null } | null;
    }

    interface SchedRun
    {
        runName?: string;
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
        // When true, the log view auto-scrolls to the newest line as messages
        // arrive. Scrolling up disengages it; the follow button re-engages it.
        private follow_logs: boolean;
        // Sticky "follow/paused" control shown over the log panel.
        private follow_bar: HTMLDivElement;
        private follow_btn: HTMLButtonElement;
        // Invoked when a QUEUE_EVENT arrives so the owner can refresh the scan queue.
        private on_queue_event?: () => void;
        // Invoked when a HISTORY_EVENT arrives so the owner can refresh scan history.
        private on_history_event?: () => void;

        constructor(beam_id: string, on_queue_event?: () => void, on_history_event?: () => void)
        {
            this.on_queue_event = on_queue_event;
            this.on_history_event = on_history_event;
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

            // Log auto-follow: on by default. A sticky pill over the log panel
            // shows the state and, when paused, re-engages following on click.
            this.follow_logs = true;
            this.follow_bar = document.createElement("div") as HTMLDivElement;
            this.follow_bar.classList.add("beamline-log-follow-bar");
            this.follow_bar.classList.add("hidden");
            this.follow_btn = document.createElement("button") as HTMLButtonElement;
            this.follow_btn.type = "button";
            this.follow_btn.classList.add("beamline-log-follow");
            this.follow_btn.addEventListener('click', () =>
            {
                this.setFollow(true);
                this.scrollLogsToBottom();
            });
            this.follow_bar.appendChild(this.follow_btn);

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
            this.content_div.appendChild(this.follow_bar);
            // Disengage follow when the user scrolls up; re-engage when they
            // scroll (or are scrolled) back to the bottom.
            this.content_div.addEventListener('scroll', () => this.onLogScroll());
            this.updateFollowBtn();

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
                case SSE_TOPIC_QUEUE_EVENT:
                {
                    // The scan (plan) queue changed; let the owner refresh it.
                    if (this.on_queue_event)
                    {
                        this.on_queue_event();
                    }
                    break;
                }
                case SSE_TOPIC_HISTORY_EVENT:
                {
                    // A scan finished; let the owner refresh the scan history.
                    if (this.on_history_event)
                    {
                        this.on_history_event();
                    }
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
            this.follow_bar.classList.toggle('hidden', !logs_active);
            this.tab_link_logs.classList.toggle('active', logs_active);
            this.tab_link_tasks.classList.toggle('active', tasks_active);

            // Opening the log panel while following snaps to the newest line.
            if (logs_active && this.follow_logs)
            {
                this.scrollLogsToBottom();
            }
        }

        // Scroll the (shared) content area to the bottom so the newest log line
        // is visible. The log/tasks panels live inside content_div, which is the
        // actual scrolling element.
        private scrollLogsToBottom(): void
        {
            this.content_div.scrollTop = this.content_div.scrollHeight;
        }

        // True when the log view is at (or within a small threshold of) the
        // bottom, i.e. the newest line is in view.
        private isLogAtBottom(): boolean
        {
            const el = this.content_div;
            return (el.scrollHeight - el.scrollTop - el.clientHeight) <= 24;
        }

        // On user scroll, follow only while pinned to the bottom. Ignored unless
        // the logs panel is the one being shown.
        private onLogScroll(): void
        {
            if (this.active_tab !== 'logs')
            {
                return;
            }
            this.setFollow(this.isLogAtBottom());
        }

        // Update the follow state and reflect it in the button.
        private setFollow(follow: boolean): void
        {
            this.follow_logs = follow;
            this.updateFollowBtn();
        }

        // Sync the follow button's label/appearance with the current state.
        private updateFollowBtn(): void
        {
            this.follow_btn.classList.toggle('following', this.follow_logs);
            this.follow_btn.innerText = this.follow_logs ? "🔒 Following" : "🔓 Paused — click to follow";
            this.follow_btn.title = this.follow_logs
                ? "Following the newest log line. Scroll up to pause."
                : "Auto-scroll paused. Click to follow the newest line again.";
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

        // Return the single <ul> that holds all log lines, creating it on first
        // use so populateLogs and appendLog share one list instead of stacking
        // a new <ul> per entry.
        private getLogList(): HTMLUListElement
        {
            let list = this.logs_div.querySelector('ul.beamline-log-list') as HTMLUListElement | null;
            if (!list)
            {
                list = document.createElement("ul") as HTMLUListElement;
                list.classList.add("beamline-log-list");
                this.logs_div.appendChild(list);
            }
            return list;
        }

        // Build one log line. Double-clicking opens a modal showing the full
        // message, pretty-printed when it parses as JSON.
        private createLogItem(item: BeamlineLog): HTMLLIElement
        {
            const list_item = document.createElement("li") as HTMLLIElement;
            list_item.classList.add("beamline-log-item");
            list_item.textContent = item.msg;
            list_item.title = "Double-click to view";
            list_item.addEventListener('dblclick', () => this.showLogModal(item));
            return list_item;
        }

        private populateLogs(nlogs: BeamlineLogs | null): void
        {
            //console.log(nlogs);
            this.logs_div.innerText = "";

            const list = this.getLogList();
            //nlogs?.reverse();
            nlogs?.forEach((item: BeamlineLog) =>
            {
                list.appendChild(this.createLogItem(item));
            });

            // Start pinned to the newest line (unless the user has paused follow).
            if (this.follow_logs)
            {
                this.scrollLogsToBottom();
            }
        }

        private appendLog(item: BeamlineLog | null): void
        {
            if(item)
            {
                const list = this.getLogList();
                list.appendChild(this.createLogItem(item));
                // Keep the newest line in view only while following; if the user
                // has scrolled up to read history, leave their position alone.
                if (this.follow_logs)
                {
                    this.scrollLogsToBottom();
                }
            }
        }

        // Pop up a modal with the full log line so long messages are readable.
        // If the message is valid JSON it is shown pretty-printed; otherwise the
        // raw text is preserved with its original whitespace.
        private showLogModal(item: BeamlineLog): void
        {
            // item.time is a unix-ish timestamp; show it when present.
            const title = item.time ? new Date(item.time).toLocaleString() : "Log Entry";
            this.showModal(title, item.msg);
        }

        // Generic detail modal shared by logs and tasks. `text` is shown
        // pretty-printed when it parses as JSON, otherwise raw with its original
        // whitespace. Closes via the × button, backdrop click, or Escape.
        private showModal(title_text: string, text: string): void
        {
            const overlay = document.createElement("div") as HTMLDivElement;
            overlay.classList.add("beamline-log-modal-overlay");

            const modal = document.createElement("div") as HTMLDivElement;
            modal.classList.add("beamline-log-modal");

            const header = document.createElement("div") as HTMLDivElement;
            header.classList.add("beamline-log-modal-header");

            const title = document.createElement("span") as HTMLSpanElement;
            title.classList.add("beamline-log-modal-title");
            title.innerText = title_text;
            header.appendChild(title);

            const close_btn = document.createElement("button") as HTMLButtonElement;
            close_btn.type = "button";
            close_btn.classList.add("beamline-log-modal-close");
            close_btn.innerText = "×"; // ×
            header.appendChild(close_btn);

            const body = document.createElement("pre") as HTMLPreElement;
            body.classList.add("beamline-log-modal-body");

            const pretty = this.tryFormatJson(text);
            if (pretty !== null)
            {
                body.textContent = pretty;
                body.classList.add("is-json");
            }
            else
            {
                body.textContent = text;
            }

            modal.appendChild(header);
            modal.appendChild(body);
            overlay.appendChild(modal);

            const close = () =>
            {
                document.removeEventListener('keydown', on_key);
                overlay.remove();
            };
            const on_key = (e: KeyboardEvent) =>
            {
                if (e.key === "Escape") { close(); }
            };

            close_btn.addEventListener('click', close);
            // Click outside the modal (on the dimmed backdrop) closes it.
            overlay.addEventListener('click', (e) =>
            {
                if (e.target === overlay) { close(); }
            });
            document.addEventListener('keydown', on_key);

            document.body.appendChild(overlay);
        }

        // Return a pretty-printed version of `text` if it parses as a JSON
        // object or array, otherwise null. Plain strings/numbers are treated as
        // "not JSON" so ordinary log lines fall through to raw display.
        private tryFormatJson(text: string): string | null
        {
            const trimmed = text.trim();
            if (!(trimmed.startsWith("{") || trimmed.startsWith("[")))
            {
                return null;
            }
            try
            {
                return JSON.stringify(JSON.parse(trimmed), null, 2);
            }
            catch (error)
            {
                return null;
            }
        }

        private populateTasks(bTasks: BeamlineTasks | null): void
        {
            //console.log(nlogs);
            this.tasks_div.innerText = "";

            let table = document.createElement("table") as HTMLTableElement;
            table.classList.add("animated-table");
            table.classList.add("beamline-tasks-table");
            // Header goes in a <thead> so the shared table styling renders a
            // proper header bar (thead th) rather than a plain first row.
            let hrow = table.createTHead().insertRow();
            ["Status", "Command", "Username", "Reply", "Start Time", "End Time"].forEach(text =>
            {
                let th = document.createElement("th");
                th.innerText = text;
                hrow.appendChild(th);
            });

            const body = table.createTBody();
            // All three queues share the same row layout; render them in order.
            bTasks?.queued.forEach((item: BeamlineTask) => body.appendChild(this.createTaskRow(item)));
            bTasks?.processing.forEach((item: BeamlineTask) => body.appendChild(this.createTaskRow(item)));
            bTasks?.done.forEach((item: BeamlineTask) => body.appendChild(this.createTaskRow(item)));

            this.tasks_div.appendChild(table);
        }

        // Build one task row. Double-clicking opens a modal showing the full
        // task, pretty-printed as JSON.
        private createTaskRow(item: BeamlineTask): HTMLTableRowElement
        {
            const row = document.createElement("tr") as HTMLTableRowElement;
            row.classList.add("beamline-task-row");
            row.title = "Double-click to view";

            row.insertCell().innerText = item.status;
            row.insertCell().innerText = item.cmd;
            row.insertCell().innerText = item.username ?? "";

            const reply_cell = row.insertCell();
            const div = document.createElement('div') as HTMLDivElement;
            div.innerText = item.reply ?? "";
            div.classList.add('scrollable');
            reply_cell.appendChild(div);

            row.insertCell().innerText = item.proc_start_time ?? "";
            row.insertCell().innerText = item.proc_end_time ?? "";

            row.addEventListener('dblclick', () =>
            {
                const title = item.cmd ? `Task: ${item.cmd}` : "Task";
                this.showModal(title, JSON.stringify(item, null, 2));
            });

            return row;
        }
    }
 
    class BeamlineScansWidget
    {
        private beamline_id: string;
        private main_div: HTMLDivElement;
        // Tab bar switching the scan panel between the live queue and history.
        private scan_tabs_div: HTMLDivElement;
        private tab_link_queue: HTMLButtonElement;
        private tab_link_history: HTMLButtonElement;
        private queue_div: HTMLDivElement;
        private history_div: HTMLDivElement;
        private queued_div: HTMLDivElement;
        // Which scan panel is shown: 'queue' (running + queued) or 'history'.
        private active_scan_tab: string;

        private available_scans: ScanApiResponse | null;

        constructor(beam_id: string)
        {
            this.main_div = document.createElement("div") as HTMLDivElement;
            this.main_div.id = "beamline-scans";
            this.main_div.classList.add("beamline-scans");

            // Tab bar: "Queue" (default) shows running + queued; "History" shows
            // completed scans.
            this.active_scan_tab = 'queue';
            this.scan_tabs_div = document.createElement("div") as HTMLDivElement;
            this.scan_tabs_div.classList.add("scan-tabs");
            this.tab_link_queue = document.createElement("button") as HTMLButtonElement;
            this.tab_link_queue.type = "button";
            this.tab_link_queue.innerText = "Queue";
            this.tab_link_queue.classList.add("scan-tab");
            this.tab_link_queue.addEventListener('click', () => this.showScanTab('queue'));
            this.tab_link_history = document.createElement("button") as HTMLButtonElement;
            this.tab_link_history.type = "button";
            this.tab_link_history.innerText = "History";
            this.tab_link_history.classList.add("scan-tab");
            this.tab_link_history.addEventListener('click', () => this.showScanTab('history'));
            this.scan_tabs_div.appendChild(this.tab_link_queue);
            this.scan_tabs_div.appendChild(this.tab_link_history);

            // Live queueserver plan queue (updated on QUEUE_EVENT).
            this.queue_div = document.createElement("div") as HTMLDivElement;
            this.queue_div.id = "beamline-scans-queue";
            this.queue_div.classList.add("beamline-scans");

            // Completed scans / history (updated on HISTORY_EVENT).
            this.history_div = document.createElement("div") as HTMLDivElement;
            this.history_div.id = "beamline-scans-history";
            this.history_div.classList.add("beamline-scans");
            this.history_div.classList.add("hidden");

            this.queued_div = document.createElement("div") as HTMLDivElement;
            this.queued_div.id = "beamline-scans-queued";
            this.queued_div.classList.add("beamline-scans");

            this.main_div.appendChild(this.scan_tabs_div);
            this.main_div.appendChild(this.queue_div);
            this.main_div.appendChild(this.history_div);
            this.main_div.appendChild(this.queued_div);

            this.available_scans = null;
            this.beamline_id = beam_id;

            this.fetchAvailableScans().then(scans =>
            {
                this.available_scans = scans;
                this.populateAvailableScans(scans);
            }
            );

            this.applyScanTabState();
            // Render the current scan queue and history, then keep them fresh via
            // QUEUE_EVENT / HISTORY_EVENT.
            this.refreshQueue();
            this.refreshHistory();
        }

        public gen_main_div(): HTMLDivElement
        {
            return this.main_div;
        }

        // Switch the active scan tab and reflect it in the DOM.
        private showScanTab(tab: string): void
        {
            this.active_scan_tab = tab;
            this.applyScanTabState();
        }

        // Show the panel for the active tab and highlight its button.
        private applyScanTabState(): void
        {
            const queue_active = this.active_scan_tab === 'queue';
            const history_active = this.active_scan_tab === 'history';
            this.queue_div.classList.toggle('hidden', !queue_active);
            this.history_div.classList.toggle('hidden', !history_active);
            this.tab_link_queue.classList.toggle('active', queue_active);
            this.tab_link_history.classList.toggle('active', history_active);
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
        // Fetch the current scan queue cached by the beamline worker (the
        // queueserver `queue_get` response, served verbatim by the backend).
        private async fetchQueuedScans(): Promise<QueueGetResponse | null>
        {
            try
            {
                const response = await fetch('/api/get_queued_scans/'+this.beamline_id);

                if (!response.ok)
                {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }

                const json_repl: QueueGetResponse = await response.json();
                return json_repl;
            }
            catch (error)
            {
                console.error('Error loading scan queue:', error);
                return null;
            }
        }

        // Re-fetch the scan queue and re-render it. Called on load and whenever a
        // QUEUE_EVENT arrives over SSE (item added/removed/reordered).
        public refreshQueue(): void
        {
            this.fetchQueuedScans().then(queue => this.populateQueuedScans(queue));
        }

        // Render the live scan queue as a table: the running item (if any)
        // followed by the ordered list of queued items.
        private populateQueuedScans(queue: QueueGetResponse | null): void
        {
            this.queue_div.innerText = "";

            if (!queue || queue.success !== true)
            {
                const msg = document.createElement("div") as HTMLDivElement;
                msg.classList.add("scan-queue-empty");
                msg.innerText = queue?.msg ? queue.msg : "No scan queue available.";
                this.queue_div.appendChild(msg);
                return;
            }

            // Currently running item, if one is present (empty object means idle).
            const running = queue.running_item as QueueItem;
            const items = Array.isArray(queue.items) ? queue.items : [];

            if (items.length === 0 && !(running && running.name))
            {
                const msg = document.createElement("div") as HTMLDivElement;
                msg.classList.add("scan-queue-empty");
                msg.innerText = "Queue is empty.";
                this.queue_div.appendChild(msg);
                return;
            }

            const table = document.createElement("table") as HTMLTableElement;
            table.classList.add("scan-queue-table");

            const hrow = table.createTHead().insertRow();
            ["#", "Plan", "Arguments", "User"].forEach(text =>
            {
                const th = document.createElement("th");
                th.innerText = text;
                hrow.appendChild(th);
            });

            const body = table.createTBody();
            if (running && running.name)
            {
                body.appendChild(this.gen_queue_row(running, "▶", true));
            }
            items.forEach((item, idx) =>
            {
                body.appendChild(this.gen_queue_row(item, String(idx + 1), false));
            });

            this.queue_div.appendChild(table);
        }

        // One row in the scan queue table: position/status marker, plan name, a
        // compact summary of its keyword arguments, and the submitting user. The
        // running item is flagged so it can be highlighted.
        private gen_queue_row(item: QueueItem, position: string, running: boolean): HTMLTableRowElement
        {
            const row = document.createElement("tr") as HTMLTableRowElement;
            row.classList.add("scan-queue-row");
            if (running)
            {
                row.classList.add("running");
            }

            const pos = row.insertCell();
            pos.classList.add("scan-queue-pos");
            pos.innerText = position;

            const name = row.insertCell();
            name.classList.add("scan-queue-name");
            name.innerText = item.name ? item.name : (item.item_type ? item.item_type : "unknown");

            const args = row.insertCell();
            args.classList.add("scan-queue-args");
            if (item.kwargs && Object.keys(item.kwargs).length > 0)
            {
                args.innerText = Object.entries(item.kwargs)
                    .map(([k, v]) => `${k}=${JSON.stringify(v)}`)
                    .join(", ");
            }
            else
            {
                args.innerText = "—";
            }

            const user = row.insertCell();
            user.classList.add("scan-queue-user");
            user.innerText = item.user ? item.user : "—";

            return row;
        }

        // Fetch the cached scan history (completed scans) served by the backend
        // from the worker's `history_get` cache.
        private async fetchScanHistory(): Promise<HistoryGetResponse | null>
        {
            try
            {
                const response = await fetch('/api/get_scan_history/'+this.beamline_id);

                if (!response.ok)
                {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }

                const json_repl: HistoryGetResponse = await response.json();
                return json_repl;
            }
            catch (error)
            {
                console.error('Error loading scan history:', error);
                return null;
            }
        }

        // Re-fetch the scan history and re-render it. Called on load and whenever a
        // HISTORY_EVENT arrives over SSE (a scan finished).
        public refreshHistory(): void
        {
            this.fetchScanHistory().then(history => this.populateScanHistory(history));
        }

        // Render the scan history as a table, newest scan first.
        private populateScanHistory(history: HistoryGetResponse | null): void
        {
            this.history_div.innerText = "";

            if (!history || history.success !== true)
            {
                const msg = document.createElement("div") as HTMLDivElement;
                msg.classList.add("scan-queue-empty");
                msg.innerText = history?.msg ? history.msg : "No scan history available.";
                this.history_div.appendChild(msg);
                return;
            }

            const items = Array.isArray(history.items) ? history.items : [];
            if (items.length === 0)
            {
                const msg = document.createElement("div") as HTMLDivElement;
                msg.classList.add("scan-queue-empty");
                msg.innerText = "No completed scans yet.";
                this.history_div.appendChild(msg);
                return;
            }

            const table = document.createElement("table") as HTMLTableElement;
            table.classList.add("scan-queue-table");

            const hrow = table.createTHead().insertRow();
            ["#", "Plan", "Arguments", "Status", "User", "Finished"].forEach(text =>
            {
                const th = document.createElement("th");
                th.innerText = text;
                hrow.appendChild(th);
            });

            const body = table.createTBody();
            // Newest scan first: history is appended chronologically, so iterate in
            // reverse and number from the most recent.
            items.slice().reverse().forEach((item, idx) =>
            {
                body.appendChild(this.gen_history_row(item, items.length - idx));
            });

            this.history_div.appendChild(table);
        }

        // One row in the history table: position, plan name, args, exit status
        // (color-coded), user, and finish time. Double-clicking opens a modal with
        // the full item (including any traceback) pretty-printed as JSON.
        private gen_history_row(item: HistoryItem, position: number): HTMLTableRowElement
        {
            const row = document.createElement("tr") as HTMLTableRowElement;
            row.classList.add("scan-queue-row");
            row.title = "Double-click to view details";

            const pos = row.insertCell();
            pos.classList.add("scan-queue-pos");
            pos.innerText = String(position);

            const name = row.insertCell();
            name.classList.add("scan-queue-name");
            name.innerText = item.name ? item.name : (item.item_type ? item.item_type : "unknown");

            const args = row.insertCell();
            args.classList.add("scan-queue-args");
            if (item.kwargs && Object.keys(item.kwargs).length > 0)
            {
                args.innerText = Object.entries(item.kwargs)
                    .map(([k, v]) => `${k}=${JSON.stringify(v)}`)
                    .join(", ");
            }
            else
            {
                args.innerText = "—";
            }

            const status = item.result?.exit_status ?? "unknown";
            const status_cell = row.insertCell();
            status_cell.classList.add("scan-history-status");
            status_cell.classList.add("status-" + status);
            status_cell.innerText = status;

            const user = row.insertCell();
            user.classList.add("scan-queue-user");
            user.innerText = item.user ? item.user : "—";

            const finished = row.insertCell();
            finished.classList.add("scan-history-time");
            finished.innerText = this.fmt_epoch(item.result?.time_stop);

            row.addEventListener('dblclick', () => this.showHistoryModal(item));

            return row;
        }

        // Format a unix-seconds timestamp for display, falling back to an em-dash.
        private fmt_epoch(secs: number | undefined): string
        {
            if (secs === undefined || !Number.isFinite(secs)) { return "—"; }
            return new Date(secs * 1000).toLocaleString();
        }

        // Pop up a modal showing a finished scan's full detail (args, result,
        // traceback) pretty-printed as JSON. Self-contained so the scans widget
        // doesn't depend on the log widget's modal.
        private showHistoryModal(item: HistoryItem): void
        {
            const overlay = document.createElement("div") as HTMLDivElement;
            overlay.classList.add("beamline-log-modal-overlay");

            const modal = document.createElement("div") as HTMLDivElement;
            modal.classList.add("beamline-log-modal");

            const header = document.createElement("div") as HTMLDivElement;
            header.classList.add("beamline-log-modal-header");

            const title = document.createElement("span") as HTMLSpanElement;
            title.classList.add("beamline-log-modal-title");
            title.innerText = item.name ? `Scan: ${item.name}` : "Scan";
            header.appendChild(title);

            const close_btn = document.createElement("button") as HTMLButtonElement;
            close_btn.type = "button";
            close_btn.classList.add("beamline-log-modal-close");
            close_btn.innerText = "×";
            header.appendChild(close_btn);

            const double_body = document.createElement("pre") as HTMLPreElement;
            double_body.classList.add("beamline-log-modal-body");
            double_body.classList.add("is-json");
            double_body.textContent = JSON.stringify(item, null, 2);

            modal.appendChild(header);
            modal.appendChild(double_body);
            overlay.appendChild(modal);

            const close = () =>
            {
                document.removeEventListener('keydown', on_key);
                overlay.remove();
            };
            const on_key = (e: KeyboardEvent) =>
            {
                if (e.key === "Escape") { close(); }
            };
            close_btn.addEventListener('click', close);
            overlay.addEventListener('click', (e) =>
            {
                if (e.target === overlay) { close(); }
            });
            document.addEventListener('keydown', on_key);

            document.body.appendChild(overlay);
        }

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
                // Header in a <thead> so the themed header bar renders.
                const hrow = table.createTHead().insertRow();
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

    // Shows a single element of a live XRF scan map as a false-color 2-D image on a
    // canvas, with dropdowns to switch between the datasets/elements available in the
    // beamline's streaming cache. Data comes from /api/get_xrf_streaming_cache as raw
    // numbers; the colormap is applied here.
    class BeamlineXrfWidget
    {
        private beamline_id: string;
        private main_div: HTMLDivElement;
        private controls_div: HTMLDivElement;
        private dataset_select: HTMLSelectElement;
        private element_select: HTMLSelectElement;
        private refresh_btn: HTMLButtonElement;
        private status_div: HTMLDivElement;
        private canvas: HTMLCanvasElement;
        // Dataset currently displayed, so element changes can re-request the same scan.
        private current_dataset: string | null;
        // Element currently displayed, so switching datasets keeps the same element.
        private current_element: string | null;

        constructor(beam_id: string)
        {
            this.beamline_id = beam_id;
            this.current_dataset = null;
            this.current_element = null;

            this.main_div = document.createElement("div") as HTMLDivElement;
            this.main_div.id = "beamline-xrf";
            this.main_div.classList.add("beamline-xrf");

            const heading = document.createElement("h3") as HTMLHeadingElement;
            heading.innerText = "Live XRF Map";
            this.main_div.appendChild(heading);

            this.controls_div = document.createElement("div") as HTMLDivElement;
            this.controls_div.classList.add("beamline-xrf-controls");

            const dataset_label = document.createElement("label") as HTMLLabelElement;
            dataset_label.innerText = "Dataset: ";
            this.dataset_select = document.createElement("select") as HTMLSelectElement;
            this.dataset_select.classList.add("beamline-xrf-select");
            // Switching dataset keeps the currently selected element (falling back to
            // the store's first element if the new dataset doesn't have it).
            this.dataset_select.addEventListener('change', () =>
            {
                this.loadMap(this.dataset_select.value, this.current_element ?? undefined);
            });
            dataset_label.appendChild(this.dataset_select);

            const element_label = document.createElement("label") as HTMLLabelElement;
            element_label.innerText = " Element: ";
            this.element_select = document.createElement("select") as HTMLSelectElement;
            this.element_select.classList.add("beamline-xrf-select");
            this.element_select.addEventListener('change', () =>
            {
                this.loadMap(this.current_dataset ?? undefined, this.element_select.value);
            });
            element_label.appendChild(this.element_select);

            this.refresh_btn = document.createElement("button") as HTMLButtonElement;
            this.refresh_btn.type = "button";
            this.refresh_btn.innerText = "Refresh";
            this.refresh_btn.addEventListener('click', () =>
            {
                // Re-pull the current selection (picks up newly streamed rows/scans).
                this.loadMap(this.current_dataset ?? undefined, this.element_select.value || undefined);
            });

            this.controls_div.appendChild(dataset_label);
            this.controls_div.appendChild(element_label);
            this.controls_div.appendChild(this.refresh_btn);
            this.main_div.appendChild(this.controls_div);

            this.status_div = document.createElement("div") as HTMLDivElement;
            this.status_div.classList.add("beamline-xrf-status");
            this.main_div.appendChild(this.status_div);

            this.canvas = document.createElement("canvas") as HTMLCanvasElement;
            this.canvas.classList.add("beamline-xrf-canvas");
            this.main_div.appendChild(this.canvas);

            this.loadMap(undefined, undefined);
        }

        public gen_main_div(): HTMLDivElement
        {
            return this.main_div;
        }

        // Fetch one element's map (optionally a specific dataset/element) and render it.
        private async loadMap(dataset?: string, element?: string): Promise<void>
        {
            this.status_div.innerText = "Loading…";
            let map = await this.fetchMap(dataset, element);
            // The remembered element may not exist in the chosen dataset (the backend
            // 404s); fall back to that dataset's default (first) element.
            if (map === null && element !== undefined)
            {
                map = await this.fetchMap(dataset, undefined);
            }
            if (map === null)
            {
                this.status_div.innerText = "No XRF data available.";
                return;
            }
            this.render(map);
        }

        private async fetchMap(dataset?: string, element?: string): Promise<XrfMap | null>
        {
            try
            {
                let url = '/api/get_xrf_streaming_cache/' + encodeURIComponent(this.beamline_id);
                const query: string[] = [];
                if (dataset) { query.push('dataset=' + encodeURIComponent(dataset)); }
                if (element) { query.push('element=' + encodeURIComponent(element)); }
                if (query.length > 0) { url += '?' + query.join('&'); }

                const response = await fetch(url);
                if (!response.ok)
                {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                return await response.json() as XrfMap;
            }
            catch (error)
            {
                console.error('Error loading XRF map:', error);
                return null;
            }
        }

        // Repopulate a <select> with `options`, selecting `selected`.
        private fill_select(select: HTMLSelectElement, options: string[], selected: string): void
        {
            select.innerText = "";
            options.forEach(opt =>
            {
                const o = document.createElement("option") as HTMLOptionElement;
                o.value = opt;
                o.textContent = opt;
                if (opt === selected) { o.selected = true; }
                select.appendChild(o);
            });
        }

        // Draw `map` onto the canvas with a false-color scale, and sync the dropdowns
        // to what is actually being shown.
        private render(map: XrfMap): void
        {
            this.current_dataset = map.dataset;
            this.current_element = map.element;
            this.fill_select(this.dataset_select, map.datasets, map.dataset);
            this.fill_select(this.element_select, map.elements, map.element);

            const w = map.width;
            const h = map.height;
            this.canvas.width = w;
            this.canvas.height = h;
            // Scale the display up while keeping pixels crisp (see beamline.css).
            this.canvas.style.width = Math.min(w * 4, 512) + "px";

            const ctx = this.canvas.getContext('2d');
            if (!ctx)
            {
                this.status_div.innerText = "Canvas not supported.";
                return;
            }

            // Normalize over the non-null values so the colormap spans the data range.
            let min = Infinity;
            let max = -Infinity;
            for (const v of map.data)
            {
                if (v !== null && Number.isFinite(v))
                {
                    if (v < min) { min = v; }
                    if (v > max) { max = v; }
                }
            }
            const span = (max > min) ? (max - min) : 1;

            const img = ctx.createImageData(w, h);
            for (let i = 0; i < map.data.length; i++)
            {
                const v = map.data[i];
                const o = i * 4;
                if (v === null || !Number.isFinite(v as number))
                {
                    // No-data pixel: dark gray so empty regions read as "not scanned".
                    img.data[o] = 32; img.data[o + 1] = 32; img.data[o + 2] = 32; img.data[o + 3] = 255;
                    continue;
                }
                const t = ((v as number) - min) / span;
                const [r, g, b] = this.colormap(t);
                img.data[o] = r; img.data[o + 1] = g; img.data[o + 2] = b; img.data[o + 3] = 255;
            }
            ctx.putImageData(img, 0, 0);

            this.status_div.innerText =
                `${map.dataset} / ${map.element} — ${w}×${h}, range ${min.toPrecision(3)}–${max.toPrecision(3)}`;
        }

        // Map t in [0,1] to an RGB triple via a compact viridis-style gradient.
        private colormap(t: number): [number, number, number]
        {
            const stops: Array<[number, number, number]> = [
                [68, 1, 84],    // dark purple
                [59, 82, 139],  // blue
                [33, 145, 140], // teal
                [94, 201, 98],  // green
                [253, 231, 37], // yellow
            ];
            const clamped = Math.max(0, Math.min(1, t));
            const scaled = clamped * (stops.length - 1);
            const idx = Math.min(Math.floor(scaled), stops.length - 2);
            const frac = scaled - idx;
            const a = stops[idx] as [number, number, number];
            const b = stops[idx + 1] as [number, number, number];
            return [
                Math.round(a[0] + (b[0] - a[0]) * frac),
                Math.round(a[1] + (b[1] - a[1]) * frac),
                Math.round(a[2] + (b[2] - a[2]) * frac),
            ];
        }
    }

    // Shows the scheduler activity currently in progress for this beamline (the one
    // whose start/end window contains "now"). Falls back to the next upcoming
    // activity for the current run, or a quiet message when nothing is scheduled.
    // The scheduler proxy is Admin/Staff-only, so for other users the fetch fails
    // and the panel simply hides itself.
    class BeamlineScheduleWidget
    {
        private beamline_id: string;
        private main_div: HTMLDivElement;
        private body_div: HTMLDivElement;

        constructor(beam_id: string)
        {
            this.beamline_id = beam_id;

            this.main_div = document.createElement("div") as HTMLDivElement;
            this.main_div.id = "beamline-schedule";
            this.main_div.classList.add("beamline-schedule");
            // Hidden until a successful fetch populates it (keeps the panel out of
            // the way for users without scheduler access or when nothing is scheduled).
            this.main_div.classList.add("hidden");

            const heading = document.createElement("h3") as HTMLHeadingElement;
            heading.innerText = "Current Scheduler Activity";
            this.main_div.appendChild(heading);

            this.body_div = document.createElement("div") as HTMLDivElement;
            this.body_div.classList.add("beamline-schedule-body");
            this.main_div.appendChild(this.body_div);

            this.loadActivity();
        }

        public gen_main_div(): HTMLDivElement
        {
            return this.main_div;
        }

        // GET a scheduler-proxy endpoint, mirroring the auth used elsewhere on the
        // page. Throws on a non-OK response so callers can hide the panel.
        private async sched_get<T>(path: string): Promise<T>
        {
            const auth_cookie: string = get_cookie('access_token');
            const headers = new Headers({
                'Accept': 'application/json',
                'Authorization': auth_cookie,
            });
            const response = await auth_fetch('/api/scheduler/' + path, { method: 'GET', headers });
            if (!response.ok)
            {
                throw new Error(`Scheduler request failed (${response.status})`);
            }
            return await response.json() as T;
        }

        // Format an ISO timestamp for display, falling back to the raw value.
        private fmt_time(value: string | null | undefined): string
        {
            if (!value) { return "—"; }
            const d = new Date(value);
            return isNaN(d.getTime()) ? value : d.toLocaleString();
        }

        // The PI for an activity: the experimenter flagged piFlag === 'Y', or the
        // first listed experimenter.
        private pi_name(act: SchedActivity): string
        {
            const experimenters = act.beamtime?.proposal?.experimenters ?? [];
            const pi = experimenters.find(e => e.piFlag === 'Y') ?? experimenters[0];
            if (!pi) { return ""; }
            return `${pi.firstName ?? ''} ${pi.lastName ?? ''}`.trim();
        }

        private user_name(act: SchedActivity): string
        {
            const user = act.user;
            if (!user) { return ""; }
            if (user.name) { return user.name; }
            const full = `${user.firstName ?? ''} ${user.lastName ?? ''}`.trim();
            return full || (user.userName ?? "");
        }

        // Fetch the current run and its activities for this beamline, then render
        // whichever activity is active now (or the next upcoming one).
        private async loadActivity(): Promise<void>
        {
            try
            {
                const run = await this.sched_get<SchedRun>('runs/current');
                const run_name = run?.runName;
                if (!run_name)
                {
                    return; // No current run; leave the panel hidden.
                }

                const activities = await this.sched_get<Array<SchedActivity>>(
                    `activity/by_run_beamline/${encodeURIComponent(run_name)}/${encodeURIComponent(this.beamline_id)}`,
                );

                const now = Date.now();
                const parse = (v: string | null | undefined) => v ? new Date(v).getTime() : NaN;

                // The activity whose [start, end] window contains now.
                const active = activities.find(a =>
                {
                    const s = parse(a.startTime);
                    const e = parse(a.endTime);
                    return !isNaN(s) && !isNaN(e) && now >= s && now <= e;
                });

                // Otherwise, the soonest activity that hasn't started yet.
                const upcoming = activities
                    .filter(a => { const s = parse(a.startTime); return !isNaN(s) && s > now; })
                    .sort((a, b) => parse(a.startTime) - parse(b.startTime))[0];

                this.render(active ?? upcoming, !!active);
                this.main_div.classList.remove('hidden');
            }
            catch (error)
            {
                // Non-staff users (403) or scheduler outages: keep the panel hidden.
                console.error('Could not load scheduler activity:', error);
            }
        }

        private render(act: SchedActivity | undefined, is_active: boolean): void
        {
            this.body_div.innerText = "";

            if (!act)
            {
                this.main_div.classList.remove('active');
                const msg = document.createElement("div") as HTMLDivElement;
                msg.classList.add("beamline-schedule-empty");
                msg.innerText = "No scheduled activity for the current run.";
                this.body_div.appendChild(msg);
                return;
            }

            // Highlight the panel when the shown activity is actually in progress.
            this.main_div.classList.toggle('active', is_active);

            const status = document.createElement("div") as HTMLDivElement;
            status.classList.add("beamline-schedule-status");
            status.innerText = is_active ? "● In progress now" : "Up next";
            this.body_div.appendChild(status);

            const rows: Array<[string, string]> = [
                ["Proposal", act.beamtime?.proposal?.gupId != null ? String(act.beamtime.proposal.gupId) : "—"],
                ["Title", act.beamtime?.proposal?.proposalTitle || "—"],
                ["PI", this.pi_name(act) || "—"],
                ["User", this.user_name(act) || "—"],
                ["Activity", act.activityName || "—"],
                ["Station", act.station?.stationName || "—"],
                ["Start", this.fmt_time(act.startTime)],
                ["End", this.fmt_time(act.endTime)],
            ];

            const table = document.createElement("table") as HTMLTableElement;
            table.classList.add("beamline-schedule-table");
            const body = table.createTBody();
            rows.forEach(([label, value]) =>
            {
                const row = body.insertRow();
                const th = document.createElement("th");
                th.innerText = label;
                row.appendChild(th);
                row.insertCell().innerText = value;
            });
            this.body_div.appendChild(table);
        }
    }

    class BeamlineWidget
    {
        private scans_widget: BeamlineScansWidget;
        private schedule_widget: BeamlineScheduleWidget;
        private refresh_plans_btn: HTMLButtonElement;
        private logs_widget: BeamlineLogWidget;
        private xrf_widget: BeamlineXrfWidget;
        private main_div: HTMLDivElement;
        private beamline_id: string;

        constructor(beam_id: string)
        {
            this.main_div = document.createElement("div") as HTMLDivElement;
            this.main_div.id = "beamline-widget";
            this.main_div.classList.add("beamline-widget");
            this.schedule_widget = new BeamlineScheduleWidget(beam_id);
            this.scans_widget = new BeamlineScansWidget(beam_id);
            this.xrf_widget = new BeamlineXrfWidget(beam_id);
            // Route scan-queue-change and scan-finished SSE events (owned by the log
            // widget's EventSource) to the scans widget so it re-renders the live
            // queue and the scan history respectively.
            this.logs_widget = new BeamlineLogWidget(
                beam_id,
                () => this.scans_widget.refreshQueue(),
                () => this.scans_widget.refreshHistory(),
            );
            this.beamline_id = beam_id;
            this.refresh_plans_btn = document.createElement("button") as HTMLButtonElement;
            this.refresh_plans_btn.innerText="Refresh Scan Plans";
            this.refresh_plans_btn.addEventListener('click', (event)=>
            { this.refresh_plans(); });

            this.main_div.appendChild(this.schedule_widget.gen_main_div())
            this.main_div.appendChild(this.refresh_plans_btn);
            this.main_div.appendChild(this.scans_widget.gen_main_div())
            this.main_div.appendChild(this.xrf_widget.gen_main_div())
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