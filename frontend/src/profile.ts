import { gen_index } from './general-helper';
import { get_user_info } from './auth';
import type { Claims } from './auth';

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

document.addEventListener('DOMContentLoaded', () =>
{
    gen_index('app', gen_profile_card());
});
