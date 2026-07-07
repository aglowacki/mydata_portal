import { gen_index } from './general-helper';
import { get_user_info, auth_fetch } from './auth';
import type { Claims } from './auth';
import { get_cookie } from './cookies';
import { show_toast } from './toast';

interface Proposal
{
    id: number;
    title: string;
    proprietaryflag: string;
    mailinflag: string;
    status: string | null;
}

function add_row(table: HTMLTableElement, label: string, value: string)
{
    const tr = document.createElement('tr');
    const th = document.createElement('th');
    th.textContent = label;
    const td = document.createElement('td');
    td.textContent = value || '—';
    tr.appendChild(th);
    tr.appendChild(td);
    table.appendChild(tr);
}

function gen_profile_card(): HTMLDivElement
{
    const card = document.createElement('div');
    card.classList.add('profile-card');

    const heading = document.createElement('h2');
    heading.textContent = 'My Profile';
    card.appendChild(heading);

    const table = document.createElement('table');
    card.appendChild(table);

    get_user_info().then((claims: Claims) =>
    {
        add_row(table, 'Username', claims.sn);
        add_row(table, 'Email', claims.mail);
        add_row(table, 'Badge / Employee ID', claims.employee_id);
        add_row(table, 'Department', claims.department);
        add_row(table, 'Employee Type', claims.employee_type);
        add_row(table, 'Access Level', claims.uac);
    }).catch(() =>
    {
        heading.textContent = 'Unable to load profile';
    });

    return card;
}

// Search the logged-in user's proposals by a dataset property. `field` must be
// one of the values the backend's /api/search_user_proposals endpoint accepts.
async function search_proposals(field: string, value: string): Promise<Array<Proposal>>
{
    const auth_cookie: string = get_cookie('access_token');
    const headers = new Headers({
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'Authorization': auth_cookie,
    });

    const url = '/api/search_user_proposals/' + encodeURIComponent(field) + '/' + encodeURIComponent(value);
    const response = await auth_fetch(url, { method: 'GET', headers: headers });
    if (!response.ok)
    {
        throw new Error(`HTTP error! status: ${response.status}`);
    }
    return await response.json() as Array<Proposal>;
}

function render_results(table: HTMLTableElement, proposals: Array<Proposal>): void
{
    table.innerHTML = '';

    if (!Array.isArray(proposals) || proposals.length === 0)
    {
        const caption = table.createCaption();
        caption.textContent = 'No matching proposals.';
        return;
    }

    const thead = table.createTHead();
    const header_row = thead.insertRow();
    ['ID', 'Title', 'Status'].forEach(text =>
    {
        const th = document.createElement('th');
        th.textContent = text;
        header_row.appendChild(th);
    });

    const tbody = table.createTBody();
    proposals.forEach(p =>
    {
        const row = tbody.insertRow();
        row.insertCell().textContent = p.id.toString();
        row.insertCell().textContent = p.title;
        row.insertCell().textContent = p.status ?? '—';
    });
}

function gen_lookup_card(): HTMLDivElement
{
    const card = document.createElement('div');
    card.classList.add('profile-card');

    const heading = document.createElement('h2');
    heading.textContent = 'Find My Proposals';
    card.appendChild(heading);

    // Shared results table, refreshed by whichever field is searched.
    const results_table = document.createElement('table');

    const fields: Array<{ key: string, label: string, placeholder: string }> =
    [
        { key: 'beamline_acronym',     label: 'Beamline Acronym',     placeholder: 'e.g. 2-ID-E' },
        { key: 'beamline_old_acronym', label: 'Beamline Old Acronym', placeholder: 'e.g. 2-ID-D' },
        { key: 'syncotron_run',        label: 'Syncotron Run Name',   placeholder: 'e.g. 2024-1' },
    ];

    fields.forEach(field =>
    {
        const row = document.createElement('div');
        row.classList.add('profile-lookup-row');

        const label = document.createElement('label');
        label.textContent = field.label + ': ';

        const input = document.createElement('input');
        input.type = 'text';
        input.placeholder = field.placeholder;

        const button = document.createElement('button');
        button.type = 'button';
        button.textContent = 'Search';

        const run_search = () =>
        {
            const value = input.value.trim();
            if (value.length === 0)
            {
                show_toast('Please enter a ' + field.label + ' to search.');
                return;
            }
            search_proposals(field.key, value)
                .then(proposals => render_results(results_table, proposals))
                .catch(error => show_toast(error.message));
        };

        button.addEventListener('click', run_search);
        input.addEventListener('keydown', (event: KeyboardEvent) =>
        {
            if (event.key === 'Enter')
            {
                event.preventDefault();
                run_search();
            }
        });

        row.appendChild(label);
        row.appendChild(input);
        row.appendChild(button);
        card.appendChild(row);
    });

    card.appendChild(results_table);
    return card;
}

document.addEventListener('DOMContentLoaded', () =>
{
    const container = document.createElement('div');
    container.appendChild(gen_profile_card());
    container.appendChild(gen_lookup_card());
    gen_index('app', container);
});
