import { get_cookie } from "./cookies";
import { show_toast } from "./toast"
import { gen_index } from './general-helper';

 interface Dataset_Struct 
 {
    id: number;
    path: string;
    acquisition_timestamp: number;
    beamline_id: number;
    syncotron_run_id: number;
    scan_type_id: number;
}

interface Proposal_Struct 
{
    id: number;
    title: string;
    proprietaryflag: string;
    mailinflag: string;
    status: string;
    datasets: Array<Dataset_Struct>;
}

class ProposalManagementApp 
{
    private proposals_json: Array<Proposal_Struct> | null = null;;

    // DOM Elements
    private proposals_table: HTMLTableElement;
    private datasets_table: HTMLTableElement;
    private admin_div: HTMLDivElement;
    private badge_label: HTMLLabelElement;
    private badge_input: HTMLInputElement;
    private update_btn: HTMLButtonElement;

    private main_div: HTMLDivElement;

    constructor() 
    {
        // admin controls
        this.admin_div = document.createElement("div") as HTMLDivElement;

        this.badge_label = document.createElement("label") as HTMLLabelElement;
        this.badge_label.innerText = "As Badge: ";

        this.badge_input = document.createElement("input") as HTMLInputElement;
        this.badge_input.id = "as_badge";

        this.update_btn = document.createElement("button") as HTMLButtonElement;
        this.update_btn.innerText = "Update";

        this.admin_div.appendChild(this.badge_label);
        this.admin_div.appendChild(this.badge_input);
        this.admin_div.appendChild(this.update_btn);

        // main layout
        this.main_div = document.createElement("div") as HTMLDivElement;
        this.main_div.id = "center";
        this.main_div.appendChild(this.admin_div);
        
        this.proposals_table = document.createElement("table") as HTMLTableElement;
        this.proposals_table.id = "proposals-table";
        this.proposals_table.className = "animated-table";
    
        this.datasets_table = document.createElement("table") as HTMLTableElement;
        this.datasets_table.id = "datasets-table";
        this.datasets_table.className = "animated-table";

        this.main_div.appendChild(this.proposals_table);
        this.main_div.appendChild(this.datasets_table);

        this.setupEventListeners();
        this.loadProposals();
    }

    public gen_main_div(): HTMLDivElement
    {
        return this.main_div;
    }

    private setupEventListeners(): void
    {
        this.update_btn.addEventListener('click', (event) =>  this.handleUpdateClick(event));
        this.proposals_table.addEventListener('click', (event) =>  this.handleRowSelection(event));
    }

    private loadProposals(): void
    {
        const resp = this.get_proposals();
        resp.then(lres =>
        {
            lres.json().then( data => 
            {
                this.proposals_json = data;
                this.fill_proposals_table(this.proposals_json);
            });
        })
        .catch(error => 
        {
            show_toast(error.message);
            //throw error;
        });
    }

    private handleUpdateClick(event: Event): void
    {
        //console.log('button clicked');
        //console.log(event);
        //console.log(event.target);
        this.update_proposals(this.badge_input.value);
    };

    private async get_proposals(): Promise<Response> 
    {
        const auth_cookie:string = get_cookie('access_token');

        const headers = new Headers({
            'Content-Type': 'application/json',
            'Accept': 'application/json',
            'Authorization': auth_cookie,
        });
        
        const requestOptions: RequestInit = {
            method: 'GET',
            headers: headers,
        };

        try 
        {
            const response = await fetch('/api/get_user_proposals', requestOptions);
            if (!response.ok) 
            {
                if(response.status == 502)
                {
                    throw new Error(`Backend Auth Serivce unreachable. ${response.status}`);
                }
                else if (response.status == 400)
                {
                    throw new Error(`Missing credentials. ${response.status}`);
                }
                else
                { 
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
            }
            else
            {
            return response;
            }
        }
        catch (error) 
        {
            console.error('There was a problem with the fetch operation:', error);
            throw error;
        }
    }

    private async get_proposals_for(badge: string): Promise<Response> 
    {
        const auth_cookie:string = get_cookie('access_token');

        const headers = new Headers({
            'Content-Type': 'application/json',
            'Accept': 'application/json',
            'Authorization': auth_cookie,
        });
        
        const requestOptions: RequestInit = {
            method: 'GET',
            headers: headers,
        };

        try 
        {
            const url = '/api/get_user_proposals_with_datasets/'+badge;
            const response = await fetch(url, requestOptions);
            if (!response.ok) 
            {
                if(response.status == 502)
                {
                    throw new Error(`Backend Auth Serivce unreachable. ${response.status}`);
                }
                else if (response.status == 400)
                {
                    throw new Error(`Missing credentials. ${response.status}`);
                }
                else
                { 
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
            }
            else
            {
            return response;
            }
        }
        catch (error) 
        {
            console.error('There was a problem with the fetch operation:', error);
            throw error;
        }

    }

    private fill_dataset_table(data: Proposal_Struct | null)
    {
        if(data === null)
        {
            console.log("Returned data is null");
            return;
        }
        
        this.datasets_table.innerHTML = "";

        if (!Array.isArray(data.datasets) || data.datasets.length === 0) 
        {
            console.log("No datasets for this proposal");
            return;
        }
        const headers = Object.keys(data.datasets[0]);
        // Create table header
        const thead = this.datasets_table.createTHead();
        headers.forEach(header => 
        {
            const th = document.createElement("th");
            th.innerText = header;
            thead.appendChild(th);
        });

        data.datasets.forEach(item => 
        {
            const row = this.datasets_table.insertRow();
            row.className = "ds-row";
            row.id = item.id.toString();

            const cell_id = row.insertCell();
            cell_id.id = 'ds-cell-id';
            cell_id.innerText = item.id.toString();

            const cell_path = row.insertCell();
            cell_path.innerText = item.path;

            const cell_aq = row.insertCell();
            cell_aq.innerText = item.acquisition_timestamp.toString();

            const cell_beam = row.insertCell();
            cell_beam.innerText = item.beamline_id.toString();

            const cell_sync = row.insertCell();
            cell_sync.innerText = item.syncotron_run_id.toString();

            const cell_st = row.insertCell();
            cell_st.innerText = item.scan_type_id.toString();

            row.offsetWidth;
            row.classList.add("visible");
        });
    }

    private handleRowSelection(event: Event) 
    {
        const clickedRow = (event.target as HTMLElement).parentNode as HTMLElement;
        if (clickedRow)
        {
            console.log(clickedRow.id);
            let numId = Number(clickedRow.id);
            this.proposals_json?.forEach(proposal => 
            {
                if(numId === proposal.id)
                {
                    this.fill_dataset_table(proposal);
                }
            });
        }
    }

    private fill_proposals_table(data: Array<Proposal_Struct> | null)
    {
        if(data === null)
        {
            console.log("Returned data is null");
            return;
        }
        
        this.proposals_table.innerHTML = "";

        if (!Array.isArray(data) || data.length === 0) 
        {
            console.log("Resply is empty array");
            return;
        }
        const headers = Object.keys(data[0]);
        // Create table header
        const thead = this.proposals_table.createTHead();
        headers.forEach(header => 
        {
            const th = document.createElement("th");
            th.innerText = header;
            thead.appendChild(th);
        });

        data.forEach(item => 
        {
            const row = this.proposals_table.insertRow();
            row.className = "ps-row";
            row.id = item.id.toString();

            const cell_id = row.insertCell();
            cell_id.id = 'ps-cell-id';
            cell_id.innerText = item.id.toString();

            const cell_title = row.insertCell();
            cell_title.innerText = item.title;

            const cell_flag = row.insertCell();
            cell_flag.innerText = item.proprietaryflag;

            const cell_mailin = row.insertCell();
            cell_mailin.innerText = item.mailinflag;

            const cell_status = row.insertCell();
            cell_status.innerText = item.status;

            const cell_num = row.insertCell();
            cell_num.innerText = item.datasets?.length.toString();

            row.offsetWidth;
            row.classList.add("visible");
        });
    }

    private update_proposals(badge: string)
    {
        const resp = this.get_proposals_for(badge);
        resp.then(lres =>
        {
            lres.json().then( data =>
            {
                this.proposals_json = data;
                this.fill_proposals_table(this.proposals_json);
                //fill_generic_table(data);
            });
        })
        .catch(error => 
        {
            show_toast(error.message);
            throw error;
        }
        );
    }

    private gen_proposals_table()
    {
        const div = document.createElement("div");
        div.id = "center";
        div.appendChild(this.admin_div);

        const table = document.createElement("table") as HTMLTableElement;
        table.id = "proposals-table";
        table.className = "animated-table";
        
        //document.body.appendChild(table);
        const resp = this.get_proposals();
        resp.then(lres =>
        {
            lres.json().then( data => 
            {
                this.proposals_json = data;
                this.fill_proposals_table(this.proposals_json);
            });
        })
        .catch(error => 
        {
            show_toast(error.message);
            //throw error;
        }
        );
        
        div.appendChild(table);

        const ds_table = document.createElement("table") as HTMLTableElement;
        ds_table.id = "datasets-table";
        ds_table.className = "animated-table";
        div.appendChild(ds_table);

        return div;
    }

};

// Initialize the application when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => 
{   
    let papp = new ProposalManagementApp();
    gen_index('app', papp.gen_main_div());
});


/*
function fill_generic_table(data: JSON)
{
    var table = document.getElementById('proposals-table') as HTMLTableElement;
    if (table == null)
    {
        console.log("Could not find table id proposals-table");
        return;
    }
    table.innerHTML = "";

    if (!Array.isArray(data) || data.length === 0) 
    {
        console.log("Resply is empty array");
        return;
    }
    const headers = Object.keys(data[0]);
    // Create table header
    const thead = table.createTHead();
    headers.forEach(header => 
    {
        const th = document.createElement("th");
        th.innerText = header;
        thead.appendChild(th);
    });

    data.forEach(item => 
    {
        const row = table.insertRow();
        row.className = "new-row";
        headers.forEach(header => 
        {
            const cell = row.insertCell();
            if (Object.prototype.toString.call(item[header]) === '[object Array]') // if array then show count
            {
                cell.innerText = item[header].length;
            }
            else
            {
                cell.innerText = item[header];
            }
        });
        row.offsetWidth; 
        row.classList.add("visible");
    });
}
*/
