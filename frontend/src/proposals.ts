import { get_cookie } from "./cookies";
import { show_toast } from "./toast"
import { gen_index } from './general-helper';
import { get_user_info } from "./auth";

 interface Dataset_Struct
 {
    id: number;
    path: string;
    acquisition_timestamp: number;
    beamline: number;
    syncotron_run: number;
    //scan_type_id: number;
}

// Datasets as returned by /api/get_proposal_datasets/{id} (beamline and
// syncotron_run are resolved to display strings by the backend).
interface Proposal_Dataset_Struct
{
    id: number;
    path: string;
    acquisition_timestamp: number;
    beamline: string;
    syncotron_run: string;
    bio_sample_id: number | null;
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
    private all_proposals_label: HTMLLabelElement;
    private all_proposals_select: HTMLSelectElement;

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

        // Admin-only dropdown listing every proposal. Hidden until we confirm
        // the current user is Admin/Staff (see initAdminControls).
        this.all_proposals_label = document.createElement("label") as HTMLLabelElement;
        this.all_proposals_label.innerText = "All Proposals: ";
        this.all_proposals_label.style.display = "none";

        this.all_proposals_select = document.createElement("select") as HTMLSelectElement;
        this.all_proposals_select.id = "all_proposals";
        this.all_proposals_select.innerHTML = '<option value="">Select a proposal...</option>';
        this.all_proposals_select.style.display = "none";

        this.admin_div.appendChild(this.badge_label);
        this.admin_div.appendChild(this.badge_input);
        this.admin_div.appendChild(this.update_btn);
        this.admin_div.appendChild(this.all_proposals_label);
        this.admin_div.appendChild(this.all_proposals_select);

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
        this.initAdminControls();
    }

    public gen_main_div(): HTMLDivElement
    {
        return this.main_div;
    }

    private setupEventListeners(): void
    {
        this.update_btn.addEventListener('click', (event) =>  this.handleUpdateClick(event));
        this.proposals_table.addEventListener('click', (event) =>  this.handleRowSelection(event));
        this.all_proposals_select.addEventListener('change', (event) =>
        {
            const item = event.target as HTMLSelectElement;
            const proposal_id = Number(item?.value);
            this.handleAllProposalsSelect(proposal_id);
        });
    }

    // Show and populate the all-proposals dropdown only for Admin/Staff users.
    private async initAdminControls(): Promise<void>
    {
        try
        {
            const claims = await get_user_info();
            if (claims.uac === 'Admin' || claims.uac === 'Staff')
            {
                this.all_proposals_label.style.display = "";
                this.all_proposals_select.style.display = "";
                await this.loadAllProposals();
            }
        }
        catch (error)
        {
            console.error('Could not determine user role; hiding admin controls:', error);
        }
    }

    private async loadAllProposals(): Promise<void>
    {
        try
        {
            const auth_cookie: string = get_cookie('access_token');
            const headers = new Headers({
                'Content-Type': 'application/json',
                'Accept': 'application/json',
                'Authorization': auth_cookie,
            });

            const response = await fetch('/api/get_all_proposals', { method: 'GET', headers: headers });
            if (!response.ok)
            {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const proposals: Array<Proposal_Struct> = await response.json();
            this.all_proposals_select.innerHTML = '<option value="">Select a proposal...</option>';
            proposals.forEach(proposal =>
            {
                const option = document.createElement("option") as HTMLOptionElement;
                option.value = proposal.id.toString();
                option.textContent = `${proposal.id} - ${proposal.title}`;
                this.all_proposals_select.appendChild(option);
            });
        }
        catch (error)
        {
            console.error('Error loading all proposals:', error);
            show_toast('Failed to load all proposals.');
        }
    }

    private handleAllProposalsSelect(proposal_id: number): void
    {
        if (!(proposal_id > 0))
        {
            this.datasets_table.innerHTML = "";
            return;
        }

        const resp = this.get_proposal_datasets(proposal_id);
        resp.then(lres =>
        {
            lres.json().then((datasets: Array<Proposal_Dataset_Struct>) =>
            {
                // Reuse the existing dataset renderer by wrapping the datasets in
                // the shape it expects. Drop bio_sample_id so the rendered
                // columns match fill_dataset_table's 5-column layout.
                const display_datasets = datasets.map(ds =>
                ({
                    id: ds.id,
                    path: ds.path,
                    acquisition_timestamp: ds.acquisition_timestamp,
                    beamline: ds.beamline,
                    syncotron_run: ds.syncotron_run,
                }));
                this.fill_dataset_table({ datasets: display_datasets } as unknown as Proposal_Struct);
            });
        })
        .catch(error =>
        {
            show_toast(error.message);
        });
    }

    private async get_proposal_datasets(proposal_id: number): Promise<Response>
    {
        const auth_cookie: string = get_cookie('access_token');
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
            const response = await fetch('/api/get_proposal_datasets/' + proposal_id, requestOptions);
            if (!response.ok)
            {
                if (response.status == 502)
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
            return response;
        }
        catch (error)
        {
            console.error('There was a problem with the fetch operation:', error);
            throw error;
        }
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
        const headers = Object.keys(data.datasets[0]!);
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
            cell_beam.innerText = item.beamline.toString();

            const cell_sync = row.insertCell();
            cell_sync.innerText = item.syncotron_run.toString();

            //const cell_st = row.insertCell();
            //cell_st.innerText = item.scan_type_id.toString();

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
        const headers = Object.keys(data[0]!);
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
