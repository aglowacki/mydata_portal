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

// Autocomplete values for the search form, as returned by
// /api/get_proposal_search_options.
interface Experimenter_Option
{
    badge: number;
    name: string;
}

interface Proposal_Search_Options
{
    runs: Array<string>;
    beamline_acronyms: Array<string>;
    experimenters: Array<Experimenter_Option>;
}

class ProposalManagementApp 
{
    private proposals_json: Array<Proposal_Struct> | null = null;;

    // Whether the current user is Admin/Staff (set once in initAdminControls).
    private is_admin: boolean = false;

    // DOM Elements
    private proposals_table: HTMLTableElement;
    private datasets_table: HTMLTableElement;
    private admin_div: HTMLDivElement;
    private badge_label: HTMLLabelElement;
    private badge_input: HTMLInputElement;
    private update_btn: HTMLButtonElement;
    private all_proposals_label: HTMLLabelElement;
    private all_proposals_select: HTMLSelectElement;

    // Search controls
    private search_div: HTMLDivElement;
    private search_run_input: HTMLInputElement;
    private search_beamline_input: HTMLInputElement;
    private search_experimenter_label: HTMLLabelElement;
    private search_experimenter_input: HTMLInputElement;
    private search_btn: HTMLButtonElement;
    private search_clear_btn: HTMLButtonElement;

    // Autocomplete option lists for the search inputs.
    private run_datalist: HTMLDataListElement;
    private beamline_datalist: HTMLDataListElement;
    private experimenter_datalist: HTMLDataListElement;

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

        // search controls
        this.search_div = document.createElement("div") as HTMLDivElement;
        this.search_div.id = "proposal-search";

        // Autocomplete option lists (populated from the DB by loadSearchOptions).
        this.run_datalist = document.createElement("datalist") as HTMLDataListElement;
        this.run_datalist.id = "run_options";
        this.beamline_datalist = document.createElement("datalist") as HTMLDataListElement;
        this.beamline_datalist.id = "beamline_options";
        this.experimenter_datalist = document.createElement("datalist") as HTMLDataListElement;
        this.experimenter_datalist.id = "experimenter_options";

        const search_run_label = document.createElement("label") as HTMLLabelElement;
        search_run_label.innerText = "Run: ";
        this.search_run_input = document.createElement("input") as HTMLInputElement;
        this.search_run_input.id = "search_run";
        this.search_run_input.placeholder = "e.g. 2024-2";
        this.search_run_input.setAttribute("list", this.run_datalist.id);

        const search_beamline_label = document.createElement("label") as HTMLLabelElement;
        search_beamline_label.innerText = " Beamline Acronym: ";
        this.search_beamline_input = document.createElement("input") as HTMLInputElement;
        this.search_beamline_input.id = "search_beamline";
        this.search_beamline_input.placeholder = "e.g. 2-ID-D";
        this.search_beamline_input.setAttribute("list", this.beamline_datalist.id);

        // Experimenter search is Admin/Staff only; hidden until confirmed.
        this.search_experimenter_label = document.createElement("label") as HTMLLabelElement;
        this.search_experimenter_label.innerText = " Experimenter: ";
        this.search_experimenter_label.style.display = "none";
        this.search_experimenter_input = document.createElement("input") as HTMLInputElement;
        this.search_experimenter_input.id = "search_experimenter";
        this.search_experimenter_input.placeholder = "badge or name";
        this.search_experimenter_input.style.display = "none";
        this.search_experimenter_input.setAttribute("list", this.experimenter_datalist.id);

        this.search_btn = document.createElement("button") as HTMLButtonElement;
        this.search_btn.innerText = "Search";

        this.search_clear_btn = document.createElement("button") as HTMLButtonElement;
        this.search_clear_btn.innerText = "Clear";

        this.search_div.appendChild(search_run_label);
        this.search_div.appendChild(this.search_run_input);
        this.search_div.appendChild(search_beamline_label);
        this.search_div.appendChild(this.search_beamline_input);
        this.search_div.appendChild(this.search_experimenter_label);
        this.search_div.appendChild(this.search_experimenter_input);
        this.search_div.appendChild(this.search_btn);
        this.search_div.appendChild(this.search_clear_btn);
        this.search_div.appendChild(this.run_datalist);
        this.search_div.appendChild(this.beamline_datalist);
        this.search_div.appendChild(this.experimenter_datalist);

        // main layout
        this.main_div = document.createElement("div") as HTMLDivElement;
        this.main_div.id = "center";
        this.main_div.appendChild(this.admin_div);
        this.main_div.appendChild(this.search_div);

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
        this.loadSearchOptions();
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

        this.search_btn.addEventListener('click', () => this.searchProposals());
        this.search_clear_btn.addEventListener('click', () => this.clearSearch());
        // Pressing Enter in any search field runs the search.
        [this.search_run_input, this.search_beamline_input, this.search_experimenter_input].forEach(input =>
        {
            input.addEventListener('keydown', (event) =>
            {
                if ((event as KeyboardEvent).key === 'Enter')
                {
                    event.preventDefault();
                    this.searchProposals();
                }
            });
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
                this.is_admin = true;
                this.all_proposals_label.style.display = "";
                this.all_proposals_select.style.display = "";
                // Admins/staff can also search by experimenter.
                this.search_experimenter_label.style.display = "";
                this.search_experimenter_input.style.display = "";
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

    // Run a proposal search using the values in the search form. Regular users
    // only ever get their own proposals; Admin/Staff search across all and may
    // also filter by experimenter.
    private async searchProposals(): Promise<void>
    {
        const run = this.search_run_input.value.trim();
        const beamline = this.search_beamline_input.value.trim();
        const experimenter = this.search_experimenter_input.value.trim();

        const params = new URLSearchParams();
        if (run.length > 0)
        {
            params.append('run', run);
        }
        if (beamline.length > 0)
        {
            params.append('beamline_acronym', beamline);
        }
        // Experimenter is only meaningful (and only sent) for Admin/Staff.
        if (this.is_admin && experimenter.length > 0)
        {
            params.append('experimenter', experimenter);
        }

        try
        {
            const auth_cookie: string = get_cookie('access_token');
            const headers = new Headers({
                'Content-Type': 'application/json',
                'Accept': 'application/json',
                'Authorization': auth_cookie,
            });

            const url = '/api/search_proposals?' + params.toString();
            const response = await fetch(url, { method: 'GET', headers: headers });
            if (!response.ok)
            {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const proposals: Array<Proposal_Struct> = await response.json();
            this.proposals_json = proposals;
            this.fill_proposals_table(this.proposals_json);
            // Clear any previously shown datasets; the search may have changed
            // which proposals are listed.
            this.datasets_table.innerHTML = "";

            if (proposals.length === 0)
            {
                show_toast('No proposals matched the search.');
            }
        }
        catch (error)
        {
            console.error('Error searching proposals:', error);
            show_toast((error as Error).message);
        }
    }

    // Load the autocomplete values for the search inputs from the database.
    // Runs and beamline acronyms are scoped to the caller's proposals (all for
    // Admin/Staff); experimenters are only returned for Admin/Staff.
    private async loadSearchOptions(): Promise<void>
    {
        try
        {
            const auth_cookie: string = get_cookie('access_token');
            const headers = new Headers({
                'Content-Type': 'application/json',
                'Accept': 'application/json',
                'Authorization': auth_cookie,
            });

            const response = await fetch('/api/get_proposal_search_options', { method: 'GET', headers: headers });
            if (!response.ok)
            {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const options: Proposal_Search_Options = await response.json();

            this.fill_datalist(this.run_datalist, options.runs.map(r => ({ value: r })));
            this.fill_datalist(this.beamline_datalist, options.beamline_acronyms.map(b => ({ value: b })));
            // Experimenter option value is the badge (what the search matches
            // exactly); the name is shown as the label.
            this.fill_datalist(this.experimenter_datalist,
                options.experimenters.map(e => ({ value: e.badge.toString(), label: e.name })));
        }
        catch (error)
        {
            console.error('Error loading search options:', error);
        }
    }

    private fill_datalist(datalist: HTMLDataListElement, entries: Array<{ value: string, label?: string }>): void
    {
        datalist.innerHTML = "";
        entries.forEach(entry =>
        {
            const option = document.createElement("option") as HTMLOptionElement;
            option.value = entry.value;
            if (entry.label !== undefined)
            {
                option.label = entry.label;
            }
            datalist.appendChild(option);
        });
    }

    // Reset the search form and reload the user's full proposal list.
    private clearSearch(): void
    {
        this.search_run_input.value = "";
        this.search_beamline_input.value = "";
        this.search_experimenter_input.value = "";
        this.datasets_table.innerHTML = "";
        this.loadProposals();
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
            let numId = Number(clickedRow.id);
            if (!(numId > 0))
            {
                return;
            }
            const proposal = this.proposals_json?.find(p => p.id === numId);
            if (proposal && Array.isArray(proposal.datasets) && proposal.datasets.length > 0)
            {
                this.fill_dataset_table(proposal);
            }
            else
            {
                // Search / plain listing responses don't embed datasets, so fetch
                // them on demand (same path the admin all-proposals dropdown uses).
                this.handleAllProposalsSelect(numId);
            }
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
            // Search / plain listing responses don't embed datasets; show '-'
            // rather than "undefined" in that case.
            cell_num.innerText = String(item.datasets?.length ?? '-');

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
