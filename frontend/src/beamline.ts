    import { gen_index } from './general-helper';
    
    interface Beamline_Log 
    {
        time: number;
        msg: string;
    }

    type Beamline_Logs = Array<Beamline_Log>;


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


    class BeamlineLogWidget
    {

        private beamline_id: string;
        private logs: Beamline_Logs | null;

        private main_div: HTMLDivElement;
        //private main_div: HTMLDivElement;

        constructor(beam_id: string)
        {
            this.main_div = document.createElement("div") as HTMLDivElement;
            this.main_div.id = "beamline-log";
            this.main_div.classList.add("beamline-log");
            this.logs = null;
            this.beamline_id = beam_id;
            this.fetcLogs().then(logs => 
            {
                //this.logs = logs
                this.populateLogs(logs);
            }
            );
        }

        public gen_main_div(): HTMLDivElement
        {
            return this.main_div;
        }

        private async fetcLogs(): Promise<Beamline_Logs | null>
        {
            try 
            {
                const response = await fetch('/api/get_beamline_log/'+this.beamline_id);

                if (!response.ok) 
                {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                
                //this.logs = await response.json();
                const json_repl: Beamline_Logs = await response.json();
                return json_repl;
            }
            catch (error) 
            {
                console.error('Error loading sample types:', error);
                return null;
                //this.showMessage('Failed to load sample types. Using default values.', 'error');
            }
        }

        private populateLogs(nlogs: Beamline_Logs | null): void
        {
            //console.log(nlogs);
            this.main_div.innerText = "";

            let htmlList = document.createElement("ul") as HTMLUListElement;
            nlogs?.forEach((item: Beamline_Log) => 
            {
                let list_item = document.createElement("li") as HTMLLIElement;
                list_item.textContent  = item.msg;
                htmlList.appendChild(list_item);
            });
            this.main_div.appendChild(htmlList);
        }
    }


    class BeamlineScansWidget
    {
        private beamline_id: string;
        private main_div: HTMLDivElement;
        private available_scans: ScanApiResponse | null;

        constructor(beam_id: string)
        {
            this.main_div = document.createElement("div") as HTMLDivElement;
            this.main_div.id = "beamline-scans";
            this.main_div.classList.add("beamline-scans");
            this.available_scans = null;
            this.beamline_id = beam_id;
            this.fetchAvailableScans().then(scans => 
            {
                //this.logs = logs
                this.populateScans(scans);
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

        private populateScans(scans: ScanApiResponse | null): void
        {
            console.log(scans);
            this.main_div.innerText = "";

            if (scans?.success === true)
            {
                let htmlList = document.createElement("ul") as HTMLUListElement;
                Object.keys(scans.plans_allowed).forEach(key => 
                {
                    //console.log(key, scans.plans_allowed[key]);
                    let list_item = document.createElement("li") as HTMLLIElement;
                    list_item.textContent  = scans.plans_allowed[key].name;
                    htmlList.appendChild(list_item);
                });
                
                this.main_div.appendChild(htmlList);
            }
        }
        
    }

    class BeamlineWidget
    {
        private scans_widget: BeamlineScansWidget;
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
            this.main_div.appendChild(this.scans_widget.gen_main_div())
            this.main_div.appendChild(this.logs_widget.gen_main_div())
        }
        
        public gen_main_div(): HTMLDivElement
        {
            return this.main_div;
        }

    }

// Initialize the application when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => 
{
    let beamline_id = 'sec0';
    let blw = new BeamlineWidget(beamline_id);
    gen_index('app', blw.gen_main_div());
});