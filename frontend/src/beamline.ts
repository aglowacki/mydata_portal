    import { gen_index } from './general-helper';
    
    interface Beamline_Log {
      time: number;
      msg: string;
    }

    class BeamlineLogWidget
    {

        private beamline_id: string;
        private logs: Array<Beamline_Log> | null;

        private main_div: HTMLDivElement;
        //private main_div: HTMLDivElement;

        constructor(beam_id: string)
        {
            this.main_div = document.createElement("div") as HTMLDivElement;
            this.main_div.id = "beamline-log";
            this.main_div.classList.add("beamline-log");
            this.logs = null;
            this.beamline_id = beam_id;
            this.loadLogs();
        }

        public gen_main_div(): HTMLDivElement
        {
            return this.main_div;
        }

        private async loadLogs(): Promise<void>
        {
            try 
            {
                const response = await fetch('/api/get_beamline_log/'+this.beamline_id);
                
                if (!response.ok) 
                {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                
                this.logs = await response.json();
                this.populateLogs();
                //this.showMessage('Sample meta data groups loaded successfully', 'success');
            }
            catch (error) 
            {
                console.error('Error loading sample types:', error);
                //this.showMessage('Failed to load sample types. Using default values.', 'error');
            }
        }

        private populateLogs(): void
        {
            this.main_div.innerText = "";


            this.main_div.appendChild
            let htmlList = document.createElement("ul") as HTMLUListElement;
            this.logs?.forEach((item) => 
            {
                let list_item = document.createElement("li") as HTMLLIElement;
                list_item.innerText = item.msg;
                htmlList.appendChild(list_item);
            });
            this.main_div.appendChild(htmlList);
        }
    }

// Initialize the application when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => 
{
    let sapp = new BeamlineLogWidget('sec0');
    gen_index('app', sapp.gen_main_div());
});