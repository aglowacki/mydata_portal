import { gen_index } from './general-helper';
import { get_cookie } from './cookies';
import { get_user_info } from './auth';

interface Proposal
{
    id: number;
    title: string;
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

    // DOM Elements
    private sample_form: HTMLFormElement;
    private sample_proposal_select: HTMLSelectElement;
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
        this.main_div = document.createElement("div") as HTMLDivElement;
        this.message_div = document.createElement("div") as HTMLDivElement;

        this.sample_form = document.createElement('form') as HTMLFormElement;
        this.sample_form.id="sampleForm";
        this.sample_form.classList.add("sample-form");

        // proposal selection input
        const div_prop = this.create_div_group("Proposal:", false);
        this.sample_proposal_select = document.createElement('select') as HTMLSelectElement;
        this.sample_proposal_select.id = 'sampleProposal';
        this.sample_proposal_select.innerHTML = '<option value="">Select a proposal...</option>';
        div_prop.appendChild(this.sample_proposal_select);
        this.sample_form.appendChild(div_prop);

        // sample selection input
        const div0 = this.create_div_group("Sample ID:", false);
        this.sample_id_input = document.createElement('input') as HTMLInputElement;
        this.sample_id_input.id = 'sampleId';
        this.sample_id_input.type = 'text';
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
        this.sample_fixative_select.innerHTML = '<option value="">Select ...</option>';
        this.setPropVisible(KEY_FIXATIVE, false);
        let added: boolean = false;
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
        
        if(fixatives_map.size === 1 && fixatives_map.has(KEY_NONE))
        {

        }
        else if (fixatives_map.size > 0)
        {
            for (let [key, value] of fixatives_map.entries())
            {
                const option = document.createElement('option') as HTMLOptionElement;
                option.value = String(value);
                option.textContent = key;
                this.sample_fixative_select.appendChild(option);   
            }
            this.setPropVisible(KEY_FIXATIVE, true);
        }
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
            this.sample_proposal_select.innerHTML = '<option value="">Select a proposal...</option>';
            proposals.forEach(proposal =>
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

        const sub_origin_id = Number(this.sample_sub_origin_select.value);
        const source_id = Number(this.sample_source_select.value);
        const cell_line = this.sample_cell_line_input.value.trim();
        const treatment = this.sample_treatment_textarea.value.trim();
        const eecc = this.sample_eecc_textarea.value.trim();
        const notes = this.sample_notes_textarea.value.trim();

        const payload = {
            id: id,
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
