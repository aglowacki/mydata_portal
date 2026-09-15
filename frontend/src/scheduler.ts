import { gen_index } from './general-helper';
import { get_user_info, auth_fetch } from './auth';
import type { Claims } from './auth';
import { get_cookie } from './cookies';
import { show_toast } from './toast';

// --- Scheduler response shapes (minimal subsets of the upstream API) --------

interface SchedRun
{
    runId?: number;
    runName?: string;
    startTime?: string;
    endTime?: string;
    version?: number;
}

interface SchedBeamline
{
    beamlineNum?: number | null;
    beamlineId?: string | null;
    beamlineName?: string | null;
}

interface SchedExperimenter
{
    badge?: string;
    firstName?: string;
    lastName?: string;
    institution?: string;
    email?: string | null;
    piFlag?: string | null;
}

interface SchedProposal
{
    gupId?: number | null;
    proposalTitle?: string | null;
    experimenters?: Array<SchedExperimenter>;
}

interface SchedBeamtime
{
    proposal?: SchedProposal;
}

interface SchedUser
{
    badgeNo?: string | null;
    firstName?: string | null;
    lastName?: string | null;
    name?: string | null;
    userName?: string | null;
    email?: string | null;
}

interface SchedActivity
{
    activityId?: number | null;
    activityName?: string | null;
    startTime?: string | null;
    endTime?: string | null;
    beamtime?: SchedBeamtime;
    station?: { stationName?: string | null } | null;
    user?: SchedUser | null;
}

// Row shape returned by findBeamtimeRequestsByRunAndBeamline.
interface SchedBeamtimeRequest
{
    beamlineId?: string | null;
    piLastName?: string | null;
    piFirstName?: string | null;
    piInstitution?: string | null;
    proposalTitle?: string | null;
    status?: string | null;
    proposalType?: string | null;
    typeDescription?: string | null;
    requestedShifts?: number | null;
    grantedShifts?: number | null;
    beamlineScheduledShifts?: number | null;
    totalScheduledShifts?: number | null;
}

// --- API helper -------------------------------------------------------------

// GET one of the backend scheduler-proxy endpoints and parse the JSON body.
async function sched_get<T>(path: string): Promise<T>
{
    const auth_cookie: string = get_cookie('access_token');
    const headers = new Headers({
        'Accept': 'application/json',
        'Authorization': auth_cookie,
    });

    const response = await auth_fetch('/api/scheduler/' + path, { method: 'GET', headers });
    if (!response.ok)
    {
        const detail = await response.text().catch(() => '');
        throw new Error(`Scheduler request failed (${response.status}): ${detail}`);
    }
    return await response.json() as T;
}

// --- Rendering helpers -------------------------------------------------------

function fmt(value: string | number | null | undefined): string
{
    if (value === null || value === undefined || value === '')
    {
        return '—';
    }
    return String(value);
}

// Format an ISO timestamp for display, falling back to the raw value.
function fmt_time(value: string | null | undefined): string
{
    if (!value)
    {
        return '—';
    }
    const d = new Date(value);
    return isNaN(d.getTime()) ? value : d.toLocaleString();
}

function pi_of(activity: SchedActivity): SchedExperimenter | undefined
{
    const experimenters = activity.beamtime?.proposal?.experimenters ?? [];
    return experimenters.find(e => e.piFlag === 'Y') ?? experimenters[0];
}

// Display name for the activity's scheduler `user`, preferring the full name and
// falling back to first/last name or the login username.
function user_of(activity: SchedActivity): string
{
    const user = activity.user;
    if (!user)
    {
        return '';
    }
    if (user.name)
    {
        return user.name;
    }
    const full = `${user.firstName ?? ''} ${user.lastName ?? ''}`.trim();
    return full || (user.userName ?? '');
}

function build_table(headers: Array<string>, rows: Array<Array<string>>): HTMLTableElement
{
    const table = document.createElement('table');

    if (rows.length === 0)
    {
        const caption = table.createCaption();
        caption.textContent = 'No results.';
        return table;
    }

    const thead = table.createTHead();
    const header_row = thead.insertRow();
    headers.forEach(text =>
    {
        const th = document.createElement('th');
        th.textContent = text;
        header_row.appendChild(th);
    });

    const tbody = table.createTBody();
    rows.forEach(cells =>
    {
        const row = tbody.insertRow();
        cells.forEach(text => { row.insertCell().textContent = text; });
    });

    return table;
}

// --- Page ------------------------------------------------------------------

function gen_scheduler_tools(): HTMLDivElement
{
    const card = document.createElement('div');
    // data-card widens the container; the activity/beamtime tables are wide.
    card.classList.add('profile-card', 'data-card');

    const heading = document.createElement('h2');
    heading.textContent = 'Beamline Scheduler';
    card.appendChild(heading);

    // Controls row: run + beamline selectors and action buttons.
    const controls = document.createElement('div');
    controls.classList.add('profile-lookup-row');

    const run_select = document.createElement('select');
    const beamline_select = document.createElement('select');

    const run_label = document.createElement('label');
    run_label.textContent = 'Run: ';
    run_label.appendChild(run_select);

    const beamline_label = document.createElement('label');
    beamline_label.textContent = 'Beamline: ';
    beamline_label.appendChild(beamline_select);

    const activities_btn = document.createElement('button');
    activities_btn.type = 'button';
    activities_btn.textContent = 'Load Activities';

    const requests_btn = document.createElement('button');
    requests_btn.type = 'button';
    requests_btn.textContent = 'Load Beamtime Requests';

    controls.appendChild(run_label);
    controls.appendChild(beamline_label);
    controls.appendChild(activities_btn);
    controls.appendChild(requests_btn);
    card.appendChild(controls);

    // Results area, replaced on each query. table-scroll lets wide tables scroll
    // horizontally instead of overflowing the card.
    const results = document.createElement('div');
    results.classList.add('table-scroll');
    card.appendChild(results);

    const set_results = (node: HTMLElement) =>
    {
        results.innerHTML = '';
        results.appendChild(node);
    };

    const placeholder = document.createElement('p');
    placeholder.textContent = 'Select a run and beamline, then load activities or beamtime requests.';
    set_results(placeholder);

    // Populate the run dropdown (all runs) and preselect the current run.
    Promise.all([
        sched_get<Array<SchedRun>>('runs/all').catch(() => [] as Array<SchedRun>),
        sched_get<SchedRun>('runs/current').catch(() => null),
    ]).then(([runs, current]) =>
    {
        const sorted = runs
            .filter(r => !!r.runName)
            .sort((a, b) => (b.runName ?? '').localeCompare(a.runName ?? ''));
        sorted.forEach(run =>
        {
            const opt = document.createElement('option');
            opt.value = run.runName ?? '';
            opt.textContent = run.runName ?? '';
            run_select.appendChild(opt);
        });
        if (current?.runName)
        {
            run_select.value = current.runName;
        }
    }).catch(error => show_toast(error.message));

    // Populate the beamline dropdown from the active beamlines list.
    sched_get<Array<SchedBeamline>>('beamlines/active').then(beamlines =>
    {
        const sorted = beamlines
            .filter(b => !!b.beamlineId)
            .sort((a, b) => (a.beamlineId ?? '').localeCompare(b.beamlineId ?? ''));
        sorted.forEach(b =>
        {
            const opt = document.createElement('option');
            opt.value = b.beamlineId ?? '';
            opt.textContent = b.beamlineName ? `${b.beamlineId} — ${b.beamlineName}` : (b.beamlineId ?? '');
            beamline_select.appendChild(opt);
        });
    }).catch(error => show_toast(error.message));

    const require_selection = (): { run: string, beamline: string } | null =>
    {
        const run = run_select.value.trim();
        const beamline = beamline_select.value.trim();
        if (!run || !beamline)
        {
            show_toast('Please choose both a run and a beamline.');
            return null;
        }
        return { run, beamline };
    };

    activities_btn.addEventListener('click', () =>
    {
        const sel = require_selection();
        if (!sel)
        {
            return;
        }
        const loading = document.createElement('p');
        loading.textContent = 'Loading activities…';
        set_results(loading);

        sched_get<Array<SchedActivity>>(
            `activity/by_run_beamline/${encodeURIComponent(sel.run)}/${encodeURIComponent(sel.beamline)}`,
        ).then(activities =>
        {
            const rows = activities.map(act =>
            {
                const pi = pi_of(act);
                const pi_name = pi ? `${pi.firstName ?? ''} ${pi.lastName ?? ''}`.trim() : '';
                return [
                    fmt(act.beamtime?.proposal?.gupId),
                    fmt(pi_name),
                    fmt(pi?.institution),
                    fmt(act.beamtime?.proposal?.proposalTitle),
                    fmt(user_of(act)),
                    fmt(act.activityName),
                    fmt(act.station?.stationName),
                    fmt_time(act.startTime),
                    fmt_time(act.endTime),
                ];
            });
            set_results(build_table(
                ['GUP', 'PI', 'Institution', 'Proposal Title', 'User', 'Activity', 'Station', 'Start', 'End'],
                rows,
            ));
        }).catch(error =>
        {
            const err = document.createElement('p');
            err.textContent = error.message;
            set_results(err);
        });
    });

    requests_btn.addEventListener('click', () =>
    {
        const sel = require_selection();
        if (!sel)
        {
            return;
        }
        const loading = document.createElement('p');
        loading.textContent = 'Loading beamtime requests…';
        set_results(loading);

        sched_get<Array<SchedBeamtimeRequest>>(
            `beamtime_requests/${encodeURIComponent(sel.run)}/${encodeURIComponent(sel.beamline)}`,
        ).then(requests =>
        {
            const rows = requests.map(req =>
            {
                const pi_name = `${req.piFirstName ?? ''} ${req.piLastName ?? ''}`.trim();
                return [
                    fmt(pi_name),
                    fmt(req.piInstitution),
                    fmt(req.proposalTitle),
                    fmt(req.typeDescription ?? req.proposalType),
                    fmt(req.status),
                    fmt(req.requestedShifts),
                    fmt(req.grantedShifts),
                    fmt(req.beamlineScheduledShifts),
                ];
            });
            set_results(build_table(
                ['PI', 'Institution', 'Proposal Title', 'Type', 'Status', 'Requested', 'Granted', 'Scheduled'],
                rows,
            ));
        }).catch(error =>
        {
            const err = document.createElement('p');
            err.textContent = error.message;
            set_results(err);
        });
    });

    return card;
}

function gen_denied_card(): HTMLDivElement
{
    const card = document.createElement('div');
    card.classList.add('profile-card');
    const heading = document.createElement('h2');
    heading.textContent = 'Access denied';
    const msg = document.createElement('p');
    msg.textContent = 'The scheduler is only available to administrators and beamline scientists.';
    card.appendChild(heading);
    card.appendChild(msg);
    return card;
}

document.addEventListener('DOMContentLoaded', () =>
{
    const container = document.createElement('div');

    // Only Admin/Staff (beamline scientists) may use the scheduler. gen_index
    // still renders the login form for unauthenticated visitors.
    get_user_info().then((claims: Claims) =>
    {
        if (claims.uac === 'Admin' || claims.uac === 'Staff')
        {
            container.appendChild(gen_scheduler_tools());
        }
        else
        {
            container.appendChild(gen_denied_card());
        }
    }).catch(() =>
    {
        // Unauthenticated — gen_index will show the login form instead.
    });

    gen_index('app', container);
});
