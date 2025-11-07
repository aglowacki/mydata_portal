import { get_cookie } from "./cookies";
import { show_toast } from "./toast"

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
    datasets: Array<Dataset_Struct>;
}

let proposals_json: Array<Proposal_Struct> | null = null;;

async function get_proposals(): Promise<Response> 
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

async function get_proposals_for(badge: string): Promise<Response> 
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

function handleRowSelection(event: Event) 
{
    const target = event.target as HTMLElement;
    const clickedRow = target.closest('tr');
    if (!clickedRow || clickedRow.parentElement?.tagName === 'THEAD') return; 
    console.log(clickedRow.id);
    /*
    */
}

function fill_ps_table(data: Array<Proposal_Struct> | null)
{
    if(data === null)
    {
        console.log("Returned data is null");
        return;
    }
    var ps_table = document.getElementById('proposals-table') as HTMLTableElement;
    if (ps_table == null)
    {
        console.log("Could not find table id proposals-table");
        return;
    }
    ps_table.innerHTML = "";

    if (!Array.isArray(data) || data.length === 0) 
    {
        console.log("Resply is empty array");
        return;
    }
    const headers = Object.keys(data[0]);
    // Create table header
    const thead = ps_table.createTHead();
    headers.forEach(header => 
    {
        const th = document.createElement("th");
        th.innerText = header;
        thead.appendChild(th);
    });

    data.forEach(item => 
    {
        const row = ps_table.insertRow();
        row.className = "new-row";
        const cell_id = row.insertCell();
        cell_id.id = 'ps-cell-id';
        cell_id.innerText = item.id.toString();
        const cell_title = row.insertCell();
        cell_title.innerText = item.title;
        const cell_num = row.insertCell();
        cell_num.innerText = item.datasets.length.toString();
        row.offsetWidth;
        row.classList.add("visible");
    });

    ps_table.addEventListener('click', handleRowSelection);
}


function update_proposals(badge: string)
{
    const resp = get_proposals_for(badge);
    resp.then(lres =>
    {
        lres.json().then( data =>
        {
            proposals_json = data;
            fill_ps_table(proposals_json);
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

function admin_controls()
{
    const div = document.createElement("div");

    const label = document.createElement("label");
    label.innerText = "As Badge: ";

    const input = document.createElement("input");
    input.id = "as_badge";
    
    const btn = document.createElement("button");
    btn.innerText = "Update";
    btn.addEventListener('click', function handleClick(event) 
    {
        console.log('button clicked');
        console.log(event);
        console.log(event.target);
        update_proposals(input.value);
    });

    div.appendChild(label);
    div.appendChild(input);
    div.appendChild(btn);

    return div;
}

export function gen_proposals_table()
{
    const div = document.createElement("div");
    div.id = "center";
    div.appendChild(admin_controls());

    const table = document.createElement("table") as HTMLTableElement;
    table.id = "proposals-table";
    table.className = "animated-table";
    
    //document.body.appendChild(table);
    const resp = get_proposals();
    resp.then(lres =>
    {
        lres.json().then( data => 
        {
            if (!Array.isArray(data) || data.length === 0) 
            {
                console.log("Resply is empty array");
                const row = table.insertRow();
                const cell = row.insertCell();
                return;
            }
            const headers = Object.keys(data[0]);
            // Create table header
            const headerRow = table.insertRow();
            headers.forEach(header => 
            {
                const th = document.createElement("th");
            
                th.innerText = header;
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

