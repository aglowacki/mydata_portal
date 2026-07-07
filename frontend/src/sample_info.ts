import { gen_index } from './general-helper';
import { get_cookie } from './cookies';
import { get_user_info } from './auth';

interface Proposal
{
    id: number;
    title: string;
}

interface ProposalDataset
{
    id: number;
    path: string;
    acquisition_timestamp: string;
    beamline: string;
    syncotron_run: string;
    bio_sample_id: number | null;
}

interface BioSample
{
    id: number;
    proposal_id: number;
    name: string;
    type_id: number;
    origin_id: number;
    sub_origin_id: number | null;
    source_id: number | null;
    thickness: number | null;
    cell_line: string | null;
    is_cancer: boolean | null;
    condition_id: number;
    treatment_details: string | null;
    fixation_id: number;
    expected_elemental_content_change: string | null;
    notes: string | null;
}

interface BioSampleType
{
    id: number;
    type_name: string;
}

interface BioSampleCondition 
{
    id: number;
    name: string;
}

interface BioSampleFixation 
{
    id: number;
    name: string;
    fixative_id: number;
}

interface BioSampleFixative 
{
    id: number;
    name: string;
}

interface SampleOrigin 
{
    id: number;
    name: string;
}

interface SampleSubOrigin 
{
    id: number;
    name: string;
}

interface SampleSource
{
    id: number;
    name: string;
}

interface SampleTypeOriginLinks
{
    id: number,
    bio_sample_type_id: number,
    origin_id: number,
    sub_origin_id: number,
}

interface SampleMetaDataGroups 
{    
    conditions: Array<BioSampleCondition>,
    fixations: Array<BioSampleFixation>,
    fixatives: Array<BioSampleFixative>,
    sample_types: Array<BioSampleType>,
    sample_origins: Array<SampleOrigin>,
    sample_sub_origins: Array<SampleSubOrigin>,
    samples_sources: Array<SampleSource>,
    sample_type_origin_links: Array<SampleTypeOriginLinks>,
}

const KEY_SAMPLE_ORIGIN: string = "Sample Origin:";
const KEY_SUB_SAMPLE_ORIGIN: string = "Sample Sub Origin:";
const KEY_SAMPLE_SOURCE: string = "Sample Source:";
const KEY_THICKNESS: string = "Thickness (microns):";
const KEY_CELL_LINE: string = "Cell Line:";
const KEY_IS_CANCER: string = "Is Cancer:";
const KEY_SAMPLE_CONDITION: string = "Sample Condition:";
const KEY_TREATMENT: string = "Treatment Details:";
const KEY_FIXATION: string = "Sample Fixation:";
const KEY_FIXATIVE: string = "Sample Fixative:";
const KEY_EECC: string = "External Elemental Content Change:";
const KEY_OTHER_NOTES: string = "Other Notes:";

const KEY_NONE: string = "None";
const KEY_CELLS: string = "Cells";
const KEYS_TISSUES: string = "Tissues";

class SampleManagementApp 
{
    //private sample_types: BioSampleType[] = [];
    private sample_meta_data_groups: SampleMetaDataGroups | null;

    // datasets fetched for the currently selected proposal
    private proposal_datasets: Array<ProposalDataset>;

    // bio samples already recorded for the currently selected proposal
    private proposal_bio_samples: Array<BioSample>;
    private sample_table_div: HTMLDivElement;

    // DOM Elements
    private sample_form: HTMLFormElement;
    private sample_proposal_select: HTMLSelectElement;
    private sample_datasets_div: HTMLDivElement;
    private sample_id_input: HTMLInputElement;
    private sample_name_input: HTMLInputElement;
    private sample_type_select: HTMLSelectElement;
    private sample_origin_select: HTMLSelectElement;
    private sample_sub_origin_select: HTMLSelectElement;
    private sample_source_select: HTMLSelectElement;
    private sample_thickness_input: HTMLInputElement;
    private sample_cell_line_input: HTMLInputElement;
    private sample_is_cancer_input: HTMLInputElement;
    private sample_condition_select: HTMLSelectElement;
    private sample_treatment_textarea: HTMLTextAreaElement;
    private sample_fixation_select: HTMLSelectElement;
    private sample_fixative_select: HTMLSelectElement;
    private sample_eecc_textarea: HTMLTextAreaElement;
    private sample_notes_textarea: HTMLTextAreaElement;
    private sample_submit_btn: HTMLButtonElement;


    private defaultHiddenOptionsStr: Array<string>;

    private message_div: HTMLDivElement;

    private main_div: HTMLDivElement;

    constructor() 
    {
        this.defaultHiddenOptionsStr = new Array();
        this.defaultHiddenOptionsStr.push(KEY_SAMPLE_ORIGIN);
        this.defaultHiddenOptionsStr.push(KEY_SUB_SAMPLE_ORIGIN);
        this.defaultHiddenOptionsStr.push(KEY_SAMPLE_SOURCE);
        this.defaultHiddenOptionsStr.push(KEY_THICKNESS);
        this.defaultHiddenOptionsStr.push(KEY_CELL_LINE);
        this.defaultHiddenOptionsStr.push(KEY_IS_CANCER);
        this.defaultHiddenOptionsStr.push(KEY_SAMPLE_CONDITION);
        this.defaultHiddenOptionsStr.push(KEY_TREATMENT);
        this.defaultHiddenOptionsStr.push(KEY_FIXATION);
        this.defaultHiddenOptionsStr.push(KEY_FIXATIVE);
        this.defaultHiddenOptionsStr.push(KEY_EECC);
        this.defaultHiddenOptionsStr.push(KEY_OTHER_NOTES);

        this.sample_meta_data_groups = null;
        this.proposal_datasets = new Array();
        this.proposal_bio_samples = new Array();
        this.main_div = document.createElement("div") as HTMLDivElement;
        this.message_div = document.createElement("div") as HTMLDivElement;

        // table listing the bio samples already recorded for the selected proposal
        this.sample_table_div = document.createElement("div") as HTMLDivElement;
        this.sample_table_div.id = 'sampleTable';
        this.sample_table_div.classList.add('sample-table');
        this.sample_table_div.textContent = 'Select a proposal to list its bio samples.';

        this.sample_form = document.createElement('form') as HTMLFormElement;
        this.sample_form.id="sampleForm";
        this.sample_form.classList.add("sample-form");

        // proposal selection input
        const div_prop = this.create_div_group("Proposal:", false);
        this.sample_proposal_select = document.createElement('select') as HTMLSelectElement;
        this.sample_proposal_select.id = 'sampleProposal';
        this.sample_proposal_select.innerHTML = '<option value="">Select a proposal...</option>';
        this.sample_proposal_select.addEventListener('change', (event) =>
        {
            let item = event.target as HTMLSelectElement;
            const value = Number(item?.value);
            this.loadProposalDatasets(value);
            this.loadProposalBioSamples(value);
        });
        div_prop.appendChild(this.sample_proposal_select);
        this.sample_form.appendChild(div_prop);

        // dataset selection - populated once a proposal is chosen
        const div_ds = this.create_div_group("Datasets:", false);
        this.sample_datasets_div = document.createElement('div') as HTMLDivElement;
        this.sample_datasets_div.id = 'sampleDatasets';
        this.sample_datasets_div.classList.add('sample-datasets-list');
        this.sample_datasets_div.textContent = 'Select a proposal to list its datasets.';
        div_ds.appendChild(this.sample_datasets_div);
        this.sample_form.appendChild(div_ds);

        // sample selection input
        const div0 = this.create_div_group("Sample ID:", false);
        this.sample_id_input = document.createElement('input') as HTMLInputElement;
        this.sample_id_input.id = 'sampleId';
        this.sample_id_input.type = 'text';
        this.sample_id_input.addEventListener('input', () => this.autoCheckDatasetsForSample());
        div0.appendChild(this.sample_id_input);
        this.sample_form.appendChild(div0);

        // sample name input
        const div1 = this.create_div_group("Sample Name:", false);
        this.sample_name_input = document.createElement('input') as HTMLInputElement;
        this.sample_name_input.id = 'sampleName';
        this.sample_name_input.type = 'text';
        div1.appendChild(this.sample_name_input);
        this.sample_form.appendChild(div1);

        // sample type input
        const div2 = this.create_div_group("Sample Type:", false);
        this.sample_type_select = document.createElement('select') as HTMLSelectElement;
        this.sample_type_select.id = 'sampleType';
        this.sample_type_select.innerHTML = '<option value="">Select a sample type...</option>';
        this.sample_type_select.addEventListener('change', (event)=>
        {
            let item = event.target as HTMLSelectElement;
            const value = Number(item?.value);            
            this.sampleTypeChanged(value);
        });
        div2.appendChild(this.sample_type_select);
        this.sample_form.appendChild(div2);

        // sample Origin input
        const div3 = this.create_div_group(KEY_SAMPLE_ORIGIN, true);
        this.sample_origin_select = document.createElement('select') as HTMLSelectElement;
        this.sample_origin_select.id = 'sampleOrigin';
        this.sample_origin_select.innerHTML = '<option value="">Select a sample origin...</option>';
        this.sample_origin_select.addEventListener('change', (event)=>
        {
            let item = event.target as HTMLSelectElement;
            const value = Number(item?.value);            
            this.sampleOriginChanged(value);
            
        });
        div3.appendChild(this.sample_origin_select);
        this.sample_form.appendChild(div3);

        const div3_1 = this.create_div_group(KEY_SUB_SAMPLE_ORIGIN, true);
        this.sample_sub_origin_select = document.createElement('select') as HTMLSelectElement;
        this.sample_sub_origin_select.id = 'sampleSubOrigin';
        this.sample_sub_origin_select.innerHTML = '<option value="">Select a sample sub origin...</option>';
        div3_1.appendChild(this.sample_sub_origin_select);
        this.sample_form.appendChild(div3_1);


        const div4 = this.create_div_group(KEY_SAMPLE_SOURCE, true);
        this.sample_source_select = document.createElement('select') as HTMLSelectElement;
        this.sample_source_select.id = 'sampleSource';
        this.sample_source_select.innerHTML = '<option value="">Select a sample source...</option>';
        div4.appendChild(this.sample_source_select);
        this.sample_form.appendChild(div4);

        const div5 = this.create_div_group(KEY_THICKNESS, true);
        this.sample_thickness_input = document.createElement('input') as HTMLInputElement;
        this.sample_thickness_input.id = 'sampleThickness';
        this.sample_thickness_input.type = 'text';
        div5.appendChild(this.sample_thickness_input);
        this.sample_form.appendChild(div5);

        const div6 = this.create_div_group(KEY_CELL_LINE, true);
        this.sample_cell_line_input = document.createElement('input') as HTMLInputElement;
        this.sample_cell_line_input.id = 'sampleCellLine';
        this.sample_cell_line_input.type = 'text';
        div6.appendChild(this.sample_cell_line_input);
        this.sample_form.appendChild(div6);

        const div7 = this.create_div_group(KEY_IS_CANCER, true);
        this.sample_is_cancer_input = document.createElement('input') as HTMLInputElement;
        this.sample_is_cancer_input.id = 'isCancer';
        this.sample_is_cancer_input.type = 'checkbox';
        div7.appendChild(this.sample_is_cancer_input);
        this.sample_form.appendChild(div7);

        const div8 = this.create_div_group(KEY_SAMPLE_CONDITION, true);
        this.sample_condition_select = document.createElement('select') as HTMLSelectElement;
        this.sample_condition_select.id = 'sampleCondition';
        this.sample_condition_select.innerHTML = '<option value="">Select a sample condition...</option>';
        this.sample_condition_select.addEventListener('change', (event)=>
        {
            let item = event.target as HTMLSelectElement;
            const value = Number(item?.value);
            this.sampleConditionChanged(value);
        });
        div8.appendChild(this.sample_condition_select);
        this.sample_form.appendChild(div8);

        const div9 = this.create_div_group(KEY_TREATMENT, true);
        this.sample_treatment_textarea = document.createElement('textarea') as HTMLTextAreaElement;
        this.sample_treatment_textarea.id = 'sampleTreatment';
        div9.appendChild(this.sample_treatment_textarea);
        this.sample_form.appendChild(div9);

        const div10 = this.create_div_group(KEY_FIXATION, true);
        this.sample_fixation_select = document.createElement('select') as HTMLSelectElement;
        this.sample_fixation_select.id = 'sampleFixation';
        this.sample_fixation_select.innerHTML = '<option value="">Select a sample fixation...</option>';
        this.sample_fixation_select.addEventListener('change', (event)=>
        {
            let item = event.target as HTMLSelectElement;
            let name: string = item?.value ?? "";
            this.sampleFixationChanged(name);
            
        });
        div10.appendChild(this.sample_fixation_select);
        this.sample_form.appendChild(div10);

        const div11 = this.create_div_group(KEY_FIXATIVE, true);
        this.sample_fixative_select = document.createElement('select') as HTMLSelectElement;
        this.sample_fixative_select.id = 'sampleFixative';
        this.sample_fixative_select.innerHTML = '<option value="">Select a sample fixative...</option>';
        div11.appendChild(this.sample_fixative_select);
        this.sample_form.appendChild(div11);

        const div12 = this.create_div_group(KEY_EECC, true);
        this.sample_eecc_textarea = document.createElement('textarea') as HTMLTextAreaElement;
        this.sample_eecc_textarea.id = 'sampleEECC';
        div12.appendChild(this.sample_eecc_textarea);
        this.sample_form.appendChild(div12);

        const div13 = this.create_div_group(KEY_OTHER_NOTES, true);
        this.sample_notes_textarea = document.createElement('textarea') as HTMLTextAreaElement;
        this.sample_notes_textarea.id = 'otherNotes';
        div13.appendChild(this.sample_notes_textarea);
        this.sample_form.appendChild(div13);

        this.sample_submit_btn = document.createElement('button') as HTMLButtonElement;
        this.sample_submit_btn.innerText = 'Submit';
        this.sample_form.appendChild(this.sample_submit_btn);

        this.main_div.appendChild(this.message_div);
        this.main_div.appendChild(this.sample_form);
        this.main_div.appendChild(this.sample_table_div);


        this.setupEventListeners();
        this.loadSampleMetaDataGroups();
        this.loadUserProposals();
    }

    public gen_main_div(): HTMLDivElement
    {
        return this.main_div;
    }

    private create_div_group(label_str: string, hidden: boolean): HTMLDivElement
    {
        const div = document.createElement('div') as HTMLDivElement;
        div.id = label_str;
        div.classList.add("sample-form-group");
        const label2 = document.createElement('label') as HTMLLabelElement;
        label2.innerText = label_str;
        div.appendChild(label2);
        if (hidden)
        {
            div.classList.add('hidden');
        }
        return div;
    }

    private setupEventListeners(): void
    {
        this.sample_form.addEventListener('submit', (e) => this.handleFormSubmit(e));
    }


    private hideAllDefault(): void
    {
        this.defaultHiddenOptionsStr.forEach(item =>
        {
            const tdiv = document.getElementById(item) as HTMLDivElement;
            tdiv.classList.add('hidden'); 
        }
        );
    }

    private setPropVisible(name: string, val: boolean): void
    {
        const tdiv = document.getElementById(name) as HTMLDivElement;
        if(val === true)
        {
            tdiv.classList.remove('hidden'); 
        }
        else
        {
            tdiv.classList.add('hidden'); 
        }
    }

    private sampleTypeChanged(id: number): void 
    {
        this.hideAllDefault();
        if(id > 0)
        {
            this.setPropVisible(KEY_SAMPLE_ORIGIN, true);
            this.sample_meta_data_groups?.sample_types.forEach(item =>
            {
                if(item.id === id && item.type_name === KEY_CELLS)
                {
                    this.setPropVisible(KEY_CELL_LINE, true);
                }
                if(item.id === id && item.type_name === KEYS_TISSUES)
                {
                    this.setPropVisible(KEY_SAMPLE_SOURCE, true);
                }
            });
            this.sample_origin_select.innerHTML = '<option value="">Select a sample origin...</option>';
            let origin_id_map: Map<number, number> = new Map();
            this.sample_meta_data_groups?.sample_type_origin_links.forEach(item =>
            {
                if(item.bio_sample_type_id === id)
                {
                    origin_id_map.set(item.origin_id, 1);
                }
            });
            let added: boolean = false;
            this.sample_meta_data_groups?.sample_origins.forEach(val => 
            {
                if (origin_id_map.has(val.id))
                {
                    const option = document.createElement('option') as HTMLOptionElement;
                    option.value = String(val.id);
                    option.textContent = val.name;
                    this.sample_origin_select.appendChild(option);
                    added = true;
                }
            });
            if (added)
            {
                this.setPropVisible(KEY_THICKNESS, true);
                this.setPropVisible(KEY_IS_CANCER, true);
                this.setPropVisible(KEY_SAMPLE_CONDITION, true);
                this.setPropVisible(KEY_FIXATION, true);
                this.setPropVisible(KEY_EECC, true);
                this.setPropVisible(KEY_OTHER_NOTES, true);
            }
        }
        else
        {
            this.hideAllDefault();
        }
    }

    private sampleOriginChanged(id: number): void 
    {
        if(id > 0)
        {
            let sample_type_id = Number(this.sample_type_select.value);
            this.sample_sub_origin_select.innerHTML = '<option value="">Select a sample sub origin...</option>';
            let sub_origin_id_map: Map<number, number> = new Map();
            this.sample_meta_data_groups?.sample_type_origin_links.forEach(item =>
            {
                if(item.bio_sample_type_id === sample_type_id && item.origin_id === id)
                {
                    sub_origin_id_map.set(item.sub_origin_id, 1);
                }
            });
            if(sub_origin_id_map.size > 0)
            {
                let added: boolean = false;
                this.sample_meta_data_groups?.sample_sub_origins.forEach(val => 
                {
                    if (sub_origin_id_map.has(val.id))
                    {
                        if(val.name === KEY_NONE)
                        {
                        
                        }
                        else
                        {
                            const option = document.createElement('option') as HTMLOptionElement;
                            option.value = String(val.id);
                            option.textContent = val.name;
                            this.sample_sub_origin_select.appendChild(option);
                            added = true;
                        }
                    }
                });

                if(added)
                {
                    this.setPropVisible(KEY_SUB_SAMPLE_ORIGIN, true);
                    
                }
            }
        }
        else
        {
            this.sample_source_select.innerHTML = '<option value="">Select a sample source...</option>';
            this.setPropVisible(KEY_SUB_SAMPLE_ORIGIN, false);
        }
    }

    private sampleConditionChanged(id: number): void 
    {
        this.setPropVisible(KEY_TREATMENT, false);
        this.sample_meta_data_groups?.conditions.forEach(val => 
        {
            if(val.name === "Treatment" && val.id === id)
            {
                this.setPropVisible(KEY_TREATMENT, true);
            }
        });
    }

    private sampleFixationChanged(name: string): void 
    {
        // Reset the fixative list before repopulating so options from a
        // previous fixation selection don't accumulate.
        this.sample_fixative_select.innerHTML = '<option value="">Select ...</option>';
        this.setPropVisible(KEY_FIXATIVE, false);
        let fixatives_map: Map<string, number> = new Map();
        this.sample_meta_data_groups?.fixations.forEach(item =>
        {
            if(item.name === name)
            {
                this.sample_meta_data_groups?.fixatives.forEach(fix =>
                {
                    if(fix.id === item.fixative_id)
                    {
                        fixatives_map.set(fix.name, fix.id);
                    }
                });
            }
        });

        for (let [key, value] of fixatives_map.entries())
        {
            const option = document.createElement('option') as HTMLOptionElement;
            option.value = String(value);
            option.textContent = key;
            this.sample_fixative_select.appendChild(option);
        }
        // if we only have 1 entry then select it automatically.
        if (fixatives_map.size == 1)
        {
          this.sample_fixative_select.selectedIndex = 1;
          //this.sample_fixative_select.dispatchEvent(new Event("change"));
        }
        this.setPropVisible(KEY_FIXATIVE, true);
    }

    private async loadSampleMetaDataGroups(): Promise<void>
    {
        try 
        {
            const response = await fetch('/api/get_bio_sample_meta_data_groups');
            
            if (!response.ok) 
            {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            
            this.sample_meta_data_groups = await response.json();
            this.populateSampleMetaDataSelect();
            this.showMessage('Sample meta data groups loaded successfully', 'success');
        }
        catch (error) 
        {
            console.error('Error loading sample types:', error);
            this.showMessage('Failed to load sample types. Using default values.', 'error');
        }
    }

    private populateSampleMetaDataSelect(): void 
    {
        // Clear existing options except the first one
        this.sample_type_select.innerHTML = '<option value="">Select a sample type...</option>';
        this.sample_origin_select.innerHTML = '<option value="">Select a sample type...</option>';
        this.sample_source_select.innerHTML = '<option value="">Select a sample type...</option>';
        this.sample_condition_select.innerHTML = '<option value="">Select a sample type...</option>';
        this.sample_fixation_select.innerHTML = '<option value="">Select a sample type...</option>';
        this.sample_fixative_select.innerHTML = '<option value="">Select a sample type...</option>';
        
        this.sample_meta_data_groups?.sample_types.forEach(sampleType => 
        {
            const option = document.createElement('option') as HTMLOptionElement;
            option.value = String(sampleType.id);
            option.textContent = sampleType.type_name;
            this.sample_type_select.appendChild(option);
        });

        this.sample_meta_data_groups?.samples_sources.forEach(item => 
        {
            const option = document.createElement('option') as HTMLOptionElement;
            option.value = String(item.id);
            option.textContent = item.name;
            this.sample_source_select.appendChild(option);
        });

        this.sample_meta_data_groups?.conditions.forEach(item => 
        {
            const option = document.createElement('option') as HTMLOptionElement;
            option.value = String(item.id);
            option.textContent = item.name;
            this.sample_condition_select.appendChild(option);
        });
        let unique_fixations_map: Map<string, number> = new Map();
        this.sample_meta_data_groups?.fixations.forEach(item => 
        {
            if(unique_fixations_map.has(item.name) === false)
            {
                unique_fixations_map.set(item.name, 1);
                const option = document.createElement('option') as HTMLOptionElement;
                option.value = item.name;
                option.textContent = item.name;
                this.sample_fixation_select.appendChild(option);
            }
        });
/*
        this.sample_meta_data_groups?.fixatives.forEach(val => 
        {
            const option = document.createElement('option') as HTMLOptionElement;
            option.value = String(val.id);
            option.textContent = val.name;
            this.sample_fixative_select.appendChild(option);
        });
  */
    }

    private async loadUserProposals(): Promise<void>
    {
        try
        {
            const auth_cookie: string = get_cookie('access_token');
            const headers = new Headers({
                'Content-Type': 'application/json',
                'Accept': 'application/json',
                'Authorization': auth_cookie,
            });

            // Admins/staff can attach samples to any proposal, so load them all;
            // everyone else only sees the proposals they are associated with.
            let url = '/api/get_user_proposals';
            try
            {
                const claims = await get_user_info();
                if (claims.uac === 'Admin' || claims.uac === 'Staff')
                {
                    url = '/api/get_all_proposals';
                }
            }
            catch (e)
            {
                console.error('Could not determine user role, defaulting to own proposals:', e);
            }

            const response = await fetch(url, { method: 'GET', headers: headers });
            if (!response.ok)
            {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const proposals: Array<Proposal> = await response.json();
            // Sort by proposal id (ascending) so the list is easy to scan;
            // sort a copy so the fetched order is left untouched.
            const sorted_proposals = proposals.slice().sort((a, b) => a.id - b.id);
            this.sample_proposal_select.innerHTML = '<option value="">Select a proposal...</option>';
            sorted_proposals.forEach(proposal =>
            {
                const option = document.createElement('option') as HTMLOptionElement;
                option.value = String(proposal.id);
                option.textContent = `${proposal.id} - ${proposal.title}`;
                this.sample_proposal_select.appendChild(option);
            });
        }
        catch (error)
        {
            console.error('Error loading proposals:', error);
            this.showMessage('Failed to load proposals. Please make sure you are logged in.', 'error');
        }
    }

    private async loadProposalDatasets(proposal_id: number): Promise<void>
    {
        if (!(proposal_id > 0))
        {
            this.proposal_datasets = [];
            this.sample_datasets_div.textContent = 'Select a proposal to list its datasets.';
            return;
        }

        try
        {
            const auth_cookie: string = get_cookie('access_token');
            const headers = new Headers({
                'Content-Type': 'application/json',
                'Accept': 'application/json',
                'Authorization': auth_cookie,
            });

            const response = await fetch('/api/get_proposal_datasets/' + proposal_id, { method: 'GET', headers: headers });
            if (!response.ok)
            {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            this.proposal_datasets = await response.json();
            this.renderDatasetCheckboxes();
        }
        catch (error)
        {
            console.error('Error loading datasets:', error);
            this.proposal_datasets = [];
            this.sample_datasets_div.textContent = 'Failed to load datasets for this proposal.';
        }
    }

    // Fetch all bio samples recorded for the proposal and render them in a table.
    private async loadProposalBioSamples(proposal_id: number): Promise<void>
    {
        if (!(proposal_id > 0))
        {
            this.proposal_bio_samples = [];
            this.sample_table_div.textContent = 'Select a proposal to list its bio samples.';
            return;
        }

        try
        {
            const auth_cookie: string = get_cookie('access_token');
            const headers = new Headers({
                'Content-Type': 'application/json',
                'Accept': 'application/json',
                'Authorization': auth_cookie,
            });

            const response = await fetch('/api/get_proposal_bio_samples/' + proposal_id, { method: 'GET', headers: headers });
            if (!response.ok)
            {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            this.proposal_bio_samples = await response.json();
            this.renderBioSampleTable();
        }
        catch (error)
        {
            console.error('Error loading bio samples:', error);
            this.proposal_bio_samples = [];
            this.sample_table_div.textContent = 'Failed to load bio samples for this proposal.';
        }
    }

    // Resolve a lookup id against a list of {id, name} rows, returning a display
    // string ('-' when the id is missing and 'unknown (id)' when unresolved).
    private lookupName(id: number | null, rows: Array<{ id: number, name: string }> | undefined): string
    {
        if (id === null || !(id > 0))
        {
            return '-';
        }
        const match = rows?.find(r => r.id === id);
        return match ? match.name : `unknown (${id})`;
    }

    // Fixations carry a name plus the fixative they use; resolve both from the
    // fixation id stored on the sample.
    private lookupFixation(fixation_id: number): { fixation: string, fixative: string }
    {
        const fixation = this.sample_meta_data_groups?.fixations.find(f => f.id === fixation_id);
        if (fixation === undefined)
        {
            return { fixation: `unknown (${fixation_id})`, fixative: '-' };
        }
        const fixative = this.sample_meta_data_groups?.fixatives.find(f => f.id === fixation.fixative_id);
        return { fixation: fixation.name, fixative: fixative ? fixative.name : '-' };
    }

    // Build (or rebuild) the table of bio samples for the selected proposal,
    // resolving the stored lookup ids to human-readable names.
    private renderBioSampleTable(): void
    {
        this.sample_table_div.innerHTML = '';
        if (this.proposal_bio_samples.length === 0)
        {
            this.sample_table_div.textContent = 'No bio samples are recorded for this proposal.';
            return;
        }

        const columns: Array<string> = [
            'ID', 'Name', 'Type', 'Origin', 'Sub Origin', 'Source',
            'Thickness (microns)', 'Cell Line', 'Is Cancer', 'Condition',
            'Treatment Details', 'Fixation', 'Fixative',
            'External Elemental Content Change', 'Notes',
        ];

        const table = document.createElement('table') as HTMLTableElement;
        table.classList.add('sample-table-grid');

        const thead = document.createElement('thead') as HTMLTableSectionElement;
        const header_row = document.createElement('tr') as HTMLTableRowElement;
        columns.forEach(col =>
        {
            const th = document.createElement('th') as HTMLTableCellElement;
            th.textContent = col;
            header_row.appendChild(th);
        });
        thead.appendChild(header_row);
        table.appendChild(thead);

        const tbody = document.createElement('tbody') as HTMLTableSectionElement;
        const g = this.sample_meta_data_groups;
        this.proposal_bio_samples.forEach(sample =>
        {
            const fix = this.lookupFixation(sample.fixation_id);
            const values: Array<string> = [
                String(sample.id),
                sample.name,
                this.lookupName(sample.type_id, g?.sample_types.map(t => ({ id: t.id, name: t.type_name }))),
                this.lookupName(sample.origin_id, g?.sample_origins),
                this.lookupName(sample.sub_origin_id, g?.sample_sub_origins),
                this.lookupName(sample.source_id, g?.samples_sources),
                sample.thickness !== null ? String(sample.thickness) : '-',
                sample.cell_line ?? '-',
                sample.is_cancer === null ? '-' : (sample.is_cancer ? 'Yes' : 'No'),
                this.lookupName(sample.condition_id, g?.conditions),
                sample.treatment_details ?? '-',
                fix.fixation,
                fix.fixative,
                sample.expected_elemental_content_change ?? '-',
                sample.notes ?? '-',
            ];

            const row = document.createElement('tr') as HTMLTableRowElement;
            row.classList.add('sample-table-row');
            values.forEach(val =>
            {
                const td = document.createElement('td') as HTMLTableCellElement;
                td.textContent = val;
                row.appendChild(td);
            });
            tbody.appendChild(row);

            // Collapsible detail row (initially hidden) that lists the datasets
            // linked to this sample. Filled lazily each time it is expanded so it
            // reflects the latest dataset assignments.
            const detail_row = document.createElement('tr') as HTMLTableRowElement;
            detail_row.classList.add('sample-detail-row', 'hidden');
            const detail_cell = document.createElement('td') as HTMLTableCellElement;
            detail_cell.colSpan = columns.length;
            detail_row.appendChild(detail_cell);
            tbody.appendChild(detail_row);

            // Clicking a row loads that sample into the form for editing and
            // toggles the expanded list of its associated datasets.
            row.addEventListener('click', () =>
            {
                const willExpand = detail_row.classList.contains('hidden');

                // Collapse any other expanded rows and clear their highlight.
                tbody.querySelectorAll('tr.sample-table-row').forEach(r => r.classList.remove('selected'));
                tbody.querySelectorAll('tr.sample-detail-row').forEach(r => r.classList.add('hidden'));

                if (willExpand)
                {
                    row.classList.add('selected');
                    this.renderSampleDatasets(detail_cell, sample);
                    detail_row.classList.remove('hidden');
                }

                this.populateFormFromSample(sample);
            });
        });
        table.appendChild(tbody);

        this.sample_table_div.appendChild(table);
    }

    // Fill a detail cell with the datasets currently linked to the given sample,
    // taken from the proposal's datasets (bio_sample_id === sample.id).
    private renderSampleDatasets(cell: HTMLTableCellElement, sample: BioSample): void
    {
        cell.innerHTML = '';

        const linked = this.proposal_datasets
            .filter(ds => ds.bio_sample_id === sample.id)
            .slice()
            .sort((a, b) => a.path.localeCompare(b.path));

        const heading = document.createElement('div') as HTMLDivElement;
        heading.classList.add('sample-detail-heading');
        heading.textContent = `Datasets for sample #${sample.id} (${sample.name})`;
        cell.appendChild(heading);

        if (linked.length === 0)
        {
            const empty = document.createElement('div') as HTMLDivElement;
            empty.textContent = 'No datasets are associated with this sample.';
            cell.appendChild(empty);
            return;
        }

        const list = document.createElement('ul') as HTMLUListElement;
        list.classList.add('sample-detail-dataset-list');
        linked.forEach(ds =>
        {
            const item = document.createElement('li') as HTMLLIElement;
            item.textContent = `${ds.path} [${ds.beamline} / ${ds.syncotron_run} / ${ds.acquisition_timestamp}]`;
            list.appendChild(item);
        });
        cell.appendChild(list);
    }

    // Fill the form with an existing sample's values so the user can edit it.
    // Fields are set in dependency order and the cascade handlers are invoked so
    // the dependent selects (origin, sub-origin, fixative) and conditional field
    // visibility match what was originally saved.
    private populateFormFromSample(sample: BioSample): void
    {
        // Setting the Sample ID switches the form to "update" mode and lets
        // autoCheckDatasetsForSample tick the datasets already linked to it.
        this.sample_id_input.value = String(sample.id);
        this.sample_name_input.value = sample.name;

        // Sample type first: it drives which origin options and conditional
        // fields become available.
        this.sample_type_select.value = String(sample.type_id);
        this.sampleTypeChanged(sample.type_id);

        // Origin (now populated by the type change) drives the sub-origin list.
        this.sample_origin_select.value = String(sample.origin_id);
        this.sampleOriginChanged(sample.origin_id);

        this.sample_sub_origin_select.value = sample.sub_origin_id !== null ? String(sample.sub_origin_id) : '';
        this.sample_source_select.value = sample.source_id !== null ? String(sample.source_id) : '';
        this.sample_thickness_input.value = sample.thickness !== null ? String(sample.thickness) : '';
        this.sample_cell_line_input.value = sample.cell_line ?? '';
        this.sample_is_cancer_input.checked = sample.is_cancer === true;

        // Condition drives whether the treatment-details field is shown.
        this.sample_condition_select.value = String(sample.condition_id);
        this.sampleConditionChanged(sample.condition_id);
        this.sample_treatment_textarea.value = sample.treatment_details ?? '';

        // The fixation <select> holds a fixation *name*; resolve the stored
        // fixation id back to (name, fixative_id) and drive the cascade so the
        // fixative list is populated before selecting the fixative.
        const fixation = this.sample_meta_data_groups?.fixations.find(f => f.id === sample.fixation_id);
        if (fixation !== undefined)
        {
            this.sample_fixation_select.value = fixation.name;
            this.sampleFixationChanged(fixation.name);
            this.sample_fixative_select.value = String(fixation.fixative_id);
        }

        this.sample_eecc_textarea.value = sample.expected_elemental_content_change ?? '';
        this.sample_notes_textarea.value = sample.notes ?? '';

        // Reset all dataset checkboxes, then tick the ones linked to this sample.
        this.proposal_datasets.forEach(ds =>
        {
            const checkbox = document.getElementById('dataset_' + ds.id) as HTMLInputElement | null;
            if (checkbox !== null)
            {
                checkbox.checked = false;
            }
        });
        this.autoCheckDatasetsForSample();

        // Bring the form into view so the user sees the loaded values.
        this.sample_form.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }

    private renderDatasetCheckboxes(): void
    {
        this.sample_datasets_div.innerHTML = '';
        if (this.proposal_datasets.length === 0)
        {
            this.sample_datasets_div.textContent = 'No datasets are associated with this proposal.';
            return;
        }

        // Show the datasets in alphabetical order by path so they are easy to
        // scan; sort a copy so the underlying fetch order is left untouched.
        const sorted_datasets = this.proposal_datasets.slice().sort((a, b) =>
            a.path.localeCompare(b.path));

        // Controls: filter the list by a regular expression and select/unselect
        // all of the rows currently visible after filtering.
        const controls = document.createElement('div') as HTMLDivElement;
        controls.classList.add('sample-dataset-controls');

        const filter_input = document.createElement('input') as HTMLInputElement;
        filter_input.type = 'text';
        filter_input.placeholder = 'Filter (regular expression)...';
        filter_input.classList.add('sample-dataset-filter');

        const select_all_btn = document.createElement('button') as HTMLButtonElement;
        select_all_btn.type = 'button';
        select_all_btn.textContent = 'Select All';

        const unselect_all_btn = document.createElement('button') as HTMLButtonElement;
        unselect_all_btn.type = 'button';
        unselect_all_btn.textContent = 'Unselect All';

        controls.appendChild(filter_input);
        controls.appendChild(select_all_btn);
        controls.appendChild(unselect_all_btn);
        this.sample_datasets_div.appendChild(controls);

        const list = document.createElement('div') as HTMLDivElement;
        list.classList.add('sample-dataset-list');
        this.sample_datasets_div.appendChild(list);

        // Keep the checkbox + row for each dataset so the controls can act on
        // only the rows currently shown by the filter.
        const rows: Array<{ row: HTMLDivElement, checkbox: HTMLInputElement, path: string }> = [];

        sorted_datasets.forEach(ds =>
        {
            const row = document.createElement('div') as HTMLDivElement;
            row.classList.add('sample-dataset-row');

            const checkbox = document.createElement('input') as HTMLInputElement;
            checkbox.type = 'checkbox';
            checkbox.value = String(ds.id);
            checkbox.id = 'dataset_' + ds.id;
            checkbox.classList.add('sample-dataset-checkbox');

            const label = document.createElement('label') as HTMLLabelElement;
            label.htmlFor = checkbox.id;
            let text = `${ds.path}`;

            //let text = `${ds.path} [${ds.beamline} / ${ds.syncotron_run} / ${ds.acquisition_timestamp}]`;
            //if (ds.bio_sample_id !== null)
            //{
            //    text += ` (currently sample #${ds.bio_sample_id})`;
            //}
            label.textContent = text;

            row.appendChild(checkbox);
            row.appendChild(label);
            list.appendChild(row);
            rows.push({ row: row, checkbox: checkbox, path: ds.path });
        });

        // Hide rows whose path does not match the filter regex. An invalid
        // regex shows everything and flags the input rather than throwing.
        const applyFilter = (): void =>
        {
            const pattern = filter_input.value.trim();
            let regex: RegExp | null = null;
            if (pattern.length > 0)
            {
                try
                {
                    regex = new RegExp(pattern, 'i');
                    filter_input.classList.remove('invalid');
                }
                catch (e)
                {
                    regex = null;
                    filter_input.classList.add('invalid');
                }
            }
            else
            {
                filter_input.classList.remove('invalid');
            }

            rows.forEach(entry =>
            {
                const matches = regex === null || regex.test(entry.path);
                entry.row.classList.toggle('hidden', !matches);
            });
        };

        // Apply to every row not currently hidden by the filter.
        const setVisibleChecked = (checked: boolean): void =>
        {
            rows.forEach(entry =>
            {
                if (!entry.row.classList.contains('hidden'))
                {
                    entry.checkbox.checked = checked;
                }
            });
        };

        filter_input.addEventListener('input', () => applyFilter());
        select_all_btn.addEventListener('click', () => setVisibleChecked(true));
        unselect_all_btn.addEventListener('click', () => setVisibleChecked(false));

        // Pre-check datasets already linked to the entered sample id (if any).
        this.autoCheckDatasetsForSample();
    }

    // Check the datasets currently linked to the sample id typed into the form,
    // so editing an existing sample starts from its current dataset selection.
    private autoCheckDatasetsForSample(): void
    {
        const id_str = this.sample_id_input.value.trim();
        if (id_str.length === 0)
        {
            return;
        }
        const sample_id = Number(id_str);
        if (!(sample_id > 0))
        {
            return;
        }
        this.proposal_datasets.forEach(ds =>
        {
            if (ds.bio_sample_id === sample_id)
            {
                const checkbox = document.getElementById('dataset_' + ds.id) as HTMLInputElement | null;
                if (checkbox !== null)
                {
                    checkbox.checked = true;
                }
            }
        });
    }

    private getSelectedDatasetIds(): Array<number>
    {
        const ids: Array<number> = [];
        this.proposal_datasets.forEach(ds =>
        {
            const checkbox = document.getElementById('dataset_' + ds.id) as HTMLInputElement | null;
            if (checkbox !== null && checkbox.checked)
            {
                ids.push(ds.id);
            }
        });
        return ids;
    }

    private handleFormSubmit(event: Event): void
    {
        event.preventDefault();

        // Basic required-field checks before sending to the backend.
        const proposal_id = Number(this.sample_proposal_select.value);
        if (!(proposal_id > 0))
        {
            this.showMessage('Please select a proposal.', 'error');
            return;
        }

        const name = this.sample_name_input.value.trim();
        if (name.length === 0)
        {
            this.showMessage('Sample Name is required.', 'error');
            return;
        }

        const type_id = Number(this.sample_type_select.value);
        if (!(type_id > 0))
        {
            this.showMessage('Please select a sample type.', 'error');
            return;
        }

        const origin_id = Number(this.sample_origin_select.value);
        if (!(origin_id > 0))
        {
            this.showMessage('Please select a sample origin.', 'error');
            return;
        }

        const condition_id = Number(this.sample_condition_select.value);
        if (!(condition_id > 0))
        {
            this.showMessage('Please select a sample condition.', 'error');
            return;
        }

        // The fixation <select> holds a fixation *name*; the fixative <select>
        // (when shown) holds a fixative id. A bio_sample_fixations row is
        // uniquely identified by (name, fixative_id), so resolve it back to its
        // primary key for the database.
        const fixation_name = this.sample_fixation_select.value;
        if (fixation_name.length === 0)
        {
            this.showMessage('Please select a sample fixation.', 'error');
            return;
        }
        const selected_fixative_id = Number(this.sample_fixative_select.value);
        let fixation_id = 0;
        this.sample_meta_data_groups?.fixations.forEach(f =>
        {
            if (f.name === fixation_name)
            {
                if (selected_fixative_id > 0)
                {
                    if (f.fixative_id === selected_fixative_id)
                    {
                        fixation_id = f.id;
                    }
                }
                else
                {
                    // No fixative was chosen (only "None" was available) - take
                    // the fixation row directly.
                    fixation_id = f.id;
                }
            }
        });
        if (!(fixation_id > 0))
        {
            this.showMessage('Please select a sample fixative.', 'error');
            return;
        }

        // Optional / conditional fields. Only visible inputs are sent; hidden
        // ones are treated as not provided (null).
        const id_str = this.sample_id_input.value.trim();
        let id: number | null = null;
        if (id_str.length > 0)
        {
            const parsed = Number(id_str);
            if (!Number.isInteger(parsed) || parsed <= 0)
            {
                this.showMessage('Sample ID must be a positive whole number (leave blank to create a new sample).', 'error');
                return;
            }
            id = parsed;
        }

        let thickness: number | null = null;
        const thickness_str = this.sample_thickness_input.value.trim();
        if (thickness_str.length > 0)
        {
            const parsed = Number(thickness_str);
            if (!Number.isInteger(parsed) || parsed < 0)
            {
                this.showMessage('Thickness must be a non-negative whole number.', 'error');
                return;
            }
            thickness = parsed;
        }

        const dataset_ids = this.getSelectedDatasetIds();
        if (dataset_ids.length === 0)
        {
            this.showMessage('Please select at least one dataset.', 'error');
            return;
        }

        const sub_origin_id = Number(this.sample_sub_origin_select.value);
        const source_id = Number(this.sample_source_select.value);
        const cell_line = this.sample_cell_line_input.value.trim();
        const treatment = this.sample_treatment_textarea.value.trim();
        const eecc = this.sample_eecc_textarea.value.trim();
        const notes = this.sample_notes_textarea.value.trim();

        const payload = {
            id: id,
            dataset_ids: dataset_ids,
            proposal_id: proposal_id,
            name: name,
            type_id: type_id,
            origin_id: origin_id,
            sub_origin_id: sub_origin_id > 0 ? sub_origin_id : null,
            source_id: source_id > 0 ? source_id : null,
            thickness: thickness,
            cell_line: cell_line.length > 0 ? cell_line : null,
            is_cancer: this.sample_is_cancer_input.checked,
            condition_id: condition_id,
            treatment_details: treatment.length > 0 ? treatment : null,
            fixation_id: fixation_id,
            expected_elemental_content_change: eecc.length > 0 ? eecc : null,
            notes: notes.length > 0 ? notes : null,
        };

        this.submitSample(payload);
    }

    private async submitSample(payload: object): Promise<void>
    {
        try
        {
            const auth_cookie: string = get_cookie('access_token');
            const headers = new Headers({
                'Content-Type': 'application/json',
                'Accept': 'application/json',
                'Authorization': auth_cookie,
            });

            const response = await fetch('/api/upsert_bio_sample', {
                method: 'POST',
                headers: headers,
                body: JSON.stringify(payload),
            });

            if (!response.ok)
            {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const result: { success: boolean, id: number | null, message: string } = await response.json();
            if (result.success)
            {
                this.showMessage(result.message, 'success');
                if (result.id !== null)
                {
                    this.sample_id_input.value = String(result.id);
                }
                // Refresh the dataset list so the "currently sample #N" notes
                // reflect the new assignments, and refresh the bio sample table
                // so it shows the newly created/updated sample.
                this.loadProposalDatasets(Number(this.sample_proposal_select.value));
                this.loadProposalBioSamples(Number(this.sample_proposal_select.value));
            }
            else
            {
                this.showMessage(result.message, 'error');
            }
        }
        catch (error)
        {
            console.error('Error submitting sample:', error);
            this.showMessage('Failed to submit sample. Please try again.', 'error');
        }
    }
/*


    private async addNewSampleType(): Promise<void>
    {
        const newTypeName = this.newSampleTypeInput.value.trim();
        
        if (!newTypeName) 
        {
            this.showMessage('Please enter a sample type name', 'error');
            return;
        }

        // Check if sample type already exists
        if (this.sampleTypes.some(type => type.name.toLowerCase() === newTypeName.toLowerCase())) 
        {
            this.showMessage('Sample type already exists', 'error');
            return;
        }

        try 
        {
            const response = await fetch('/api/sample_types', 
            {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ name: newTypeName })
            });

            if (!response.ok) 
            {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const newSampleType: SampleType = await response.json();
            this.sampleTypes.push(newSampleType);
            this.populateSampleTypeSelect();
            this.newSampleTypeInput.value = '';
            this.showMessage(`Sample type "${newTypeName}" added successfully`, 'success');
        } 
        catch (error) 
        {
            console.error('Error adding sample type:', error);
            
            // Fallback for development/testing
            const newId = (this.sampleTypes.length + 1).toString();
            const newSampleType: SampleType = { id: newId, name: newTypeName };
            this.sampleTypes.push(newSampleType);
            this.populateSampleTypeSelect();
            this.newSampleTypeInput.value = '';
            this.showMessage(`Sample type "${newTypeName}" added locally (backend unavailable)`, 'success');
        }
    }

    private handleFormSubmit(event: Event): void {
        event.preventDefault();
        
        const formData = new FormData(this.sampleForm);
        const sample: Sample = {
            sampleId: formData.get('sampleId') as string,
            sampleName: formData.get('sampleName') as string,
            sampleType: formData.get('sampleType') as string,
            isGood: formData.has('isGood'),
            otherNotes: formData.get('otherNotes') as string || ''
        };

        if (this.validateSample(sample)) {
            this.submitSample(sample);
        }
    }

    private validateSample(sample: Sample): boolean {
        if (!sample.sampleId.trim()) {
            this.showMessage('Sample ID is required', 'error');
            return false;
        }

        if (!sample.sampleName.trim()) {
            this.showMessage('Sample Name is required', 'error');
            return false;
        }

        if (!sample.sampleType) {
            this.showMessage('Sample Type is required', 'error');
            return false;
        }

        return true;
    }

    private async submitSample(sample: Sample): Promise<void> {
        try {
            // Here you would typically send the sample data to your backend
            console.log('Submitting sample:', sample);
            
            // Simulate API call
            await new Promise(resolve => setTimeout(resolve, 1000));
            
            this.showMessage('Sample submitted successfully!', 'success');
            this.sampleForm.reset();
        } catch (error) {
            console.error('Error submitting sample:', error);
            this.showMessage('Failed to submit sample. Please try again.', 'error');
        }
    }
*/
    private showMessage(text: string, type: 'success' | 'error'): void 
    {
        this.message_div.textContent = text;
        this.message_div.className = `message ${type}`;
        this.message_div.classList.remove('hidden');
        
        // Auto-hide message after 5 seconds
        setTimeout(() => 
        {
            this.message_div.classList.add('hidden');
        }, 5000);
    }
        
}

// Initialize the application when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => 
{
    let sapp = new SampleManagementApp();
    gen_index('app', sapp.gen_main_div());
});
