import { gen_index } from './general-helper';

interface BioSampleType 
{
    id: string;
    type_name: string;
}

interface BioSampleCondition 
{
    id: string;
    name: string;
}

interface BioSampleFixation 
{
    id: string;
    name: string;
    fixative_id: number;
}

interface BioSampleFixative 
{
    id: string;
    name: string;
}

interface SampleOrigin 
{
    id: string;
    name: string;
}

interface SampleSubOrigin 
{
    id: string;
    name: string;
}

interface SampleSource
{
    id: string;
    name: string;
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
}

class SampleManagementApp 
{
    //private sample_types: BioSampleType[] = [];
    private sample_meta_data_groups: SampleMetaDataGroups | null;

    // DOM Elements
    private sample_form: HTMLFormElement;
    private sample_id_input: HTMLInputElement;
    private sample_name_input: HTMLInputElement;
    private sample_type_select: HTMLSelectElement;
    private sample_origin_select: HTMLSelectElement;
    private sample_source_select: HTMLSelectElement;
    private sample_thickness_input: HTMLInputElement;
    private sample_cell_line_input: HTMLInputElement;
    private sample_is_cancer_input: HTMLInputElement;
    private sample_condition_select: HTMLSelectElement;
    private sample_treatment_input: HTMLInputElement;
    private sample_fixation_select: HTMLSelectElement;
    private sample_fixative_select: HTMLSelectElement;
    private sample_eecc_textarea: HTMLTextAreaElement;
    private sample_notes_textarea: HTMLTextAreaElement;
    private sample_submit_btn: HTMLButtonElement;

    private message_div: HTMLDivElement;

    private main_div: HTMLDivElement;

    constructor() 
    {
        this.sample_meta_data_groups = null;
        this.main_div = document.createElement("div") as HTMLDivElement;
        this.message_div = document.createElement("div") as HTMLDivElement;

        this.sample_form = document.createElement('form') as HTMLFormElement;
        this.sample_form.id="sampleForm";
        this.sample_form.classList.add("sample-form");

        // sample selection input
        const div0 = this.create_div_group("Sample ID:");
        this.sample_id_input = document.createElement('input') as HTMLInputElement;
        this.sample_id_input.id = 'sampleId';
        this.sample_id_input.type = 'text';
        div0.appendChild(this.sample_id_input);
        this.sample_form.appendChild(div0);

        // sample name input
        const div1 = this.create_div_group("Sample Name:");
        this.sample_name_input = document.createElement('input') as HTMLInputElement;
        this.sample_name_input.id = 'sampleName';
        this.sample_name_input.type = 'text';
        div1.appendChild(this.sample_name_input);
        this.sample_form.appendChild(div1);

        // sample type input
        const div2 = this.create_div_group("Sample Type:");
        this.sample_type_select = document.createElement('select') as HTMLSelectElement;
        this.sample_type_select.id = 'sampleType';
        this.sample_type_select.innerHTML = '<option value="">Select a sample type...</option>';
        div2.appendChild(this.sample_type_select);
        this.sample_form.appendChild(div2);

        // sample Origin input
        const div3 = this.create_div_group("Sample Origin:");
        this.sample_origin_select = document.createElement('select') as HTMLSelectElement;
        this.sample_origin_select.id = 'sampleOrigin';
        this.sample_origin_select.innerHTML = '<option value="">Select a sample origin...</option>';
        div3.appendChild(this.sample_origin_select);
        this.sample_form.appendChild(div3);

        const div4 = this.create_div_group("Sample Source:");
        this.sample_source_select = document.createElement('select') as HTMLSelectElement;
        this.sample_source_select.id = 'sampleSource';
        this.sample_source_select.innerHTML = '<option value="">Select a sample source...</option>';
        div4.appendChild(this.sample_source_select);
        this.sample_form.appendChild(div4);

        const div5 = this.create_div_group("Thickness (microns):");
        this.sample_thickness_input = document.createElement('input') as HTMLInputElement;
        this.sample_thickness_input.id = 'sampleThickness';
        this.sample_thickness_input.type = 'text';
        div5.appendChild(this.sample_thickness_input);
        this.sample_form.appendChild(div5);

        const div6 = this.create_div_group("Cell Line:");
        this.sample_cell_line_input = document.createElement('input') as HTMLInputElement;
        this.sample_cell_line_input.id = 'sampleCellLine';
        this.sample_cell_line_input.type = 'text';
        div6.appendChild(this.sample_cell_line_input);
        this.sample_form.appendChild(div6);

        const div7 = this.create_div_group("Is Cancer:");
        this.sample_is_cancer_input = document.createElement('input') as HTMLInputElement;
        this.sample_is_cancer_input.id = 'isCancer';
        this.sample_is_cancer_input.type = 'checkbox';
        div7.appendChild(this.sample_is_cancer_input);
        this.sample_form.appendChild(div7);

        const div8 = this.create_div_group("Sample Condition:");
        this.sample_condition_select = document.createElement('select') as HTMLSelectElement;
        this.sample_condition_select.id = 'sampleCondition';
        this.sample_condition_select.innerHTML = '<option value="">Select a sample condition...</option>';
        div8.appendChild(this.sample_condition_select);
        this.sample_form.appendChild(div8);

        const div9 = this.create_div_group("Treatment Details:");
        this.sample_treatment_input = document.createElement('input') as HTMLInputElement;
        this.sample_treatment_input.id = 'sampleTreatment';
        this.sample_treatment_input.type = 'text';
        div9.appendChild(this.sample_treatment_input);
        this.sample_form.appendChild(div9);

        const div10 = this.create_div_group("Sample Fixation:");
        this.sample_fixation_select = document.createElement('select') as HTMLSelectElement;
        this.sample_fixation_select.id = 'sampleFixation';
        this.sample_fixation_select.innerHTML = '<option value="">Select a sample fixation...</option>';
        div10.appendChild(this.sample_fixation_select);
        this.sample_form.appendChild(div10);

        const div11 = this.create_div_group("Sample Fixative:");
        this.sample_fixative_select = document.createElement('select') as HTMLSelectElement;
        this.sample_fixative_select.id = 'sampleFixative';
        this.sample_fixative_select.innerHTML = '<option value="">Select a sample fixative...</option>';
        div11.appendChild(this.sample_fixative_select);
        this.sample_form.appendChild(div11);

        const div12 = this.create_div_group("External Elemental Content Change:");
        this.sample_eecc_textarea = document.createElement('textarea') as HTMLTextAreaElement;
        this.sample_eecc_textarea.id = 'sampleEECC';
        div12.appendChild(this.sample_eecc_textarea);
        this.sample_form.appendChild(div12);

        const div13 = this.create_div_group("Other Notes:");
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
    }

    public gen_main_div(): HTMLDivElement
    {
        return this.main_div;
    }

    private create_div_group(label_str: string): HTMLDivElement
    {
        const div = document.createElement('div') as HTMLDivElement;
        div.classList.add("sample-form-group");
        const label2 = document.createElement('label') as HTMLLabelElement;
        label2.innerText = label_str;
        div.appendChild(label2);
        return div;
    }

    private setupEventListeners(): void 
    {
        //this.adminToggle.addEventListener('click', () => this.toggleAdminMode());
        //this.addSampleTypeButton.addEventListener('click', () => this.addNewSampleType());
        //this.sampleForm.addEventListener('submit', (e) => this.handleFormSubmit(e));    
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
            option.value = sampleType.id;
            option.textContent = sampleType.type_name;
            this.sample_type_select.appendChild(option);
        });

        this.sample_meta_data_groups?.sample_origins.forEach(val => 
        {
            const option = document.createElement('option') as HTMLOptionElement;
            option.value = val.id;
            option.textContent = val.name;
            this.sample_origin_select.appendChild(option);
        });

        this.sample_meta_data_groups?.samples_sources.forEach(val => 
        {
            const option = document.createElement('option') as HTMLOptionElement;
            option.value = val.id;
            option.textContent = val.name;
            this.sample_source_select.appendChild(option);
        });

        this.sample_meta_data_groups?.conditions.forEach(val => 
        {
            const option = document.createElement('option') as HTMLOptionElement;
            option.value = val.id;
            option.textContent = val.name;
            this.sample_condition_select.appendChild(option);
        });

        this.sample_meta_data_groups?.fixations.forEach(val => 
        {
            const option = document.createElement('option') as HTMLOptionElement;
            option.value = val.id;
            option.textContent = val.name;
            this.sample_fixation_select.appendChild(option);
        });

        this.sample_meta_data_groups?.fixatives.forEach(val => 
        {
            const option = document.createElement('option') as HTMLOptionElement;
            option.value = val.id;
            option.textContent = val.name;
            this.sample_fixative_select.appendChild(option);
        });
    }
/*
    private async loadSampleTypes(): Promise<void> 
    {
        try 
        {
            const response = await fetch('/api/bio_sample_types');
            
            if (!response.ok) 
            {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            
            this.sample_types = await response.json();
            this.populateSampleTypeSelect();
            this.showMessage('Sample types loaded successfully', 'success');
        }
        catch (error) 
        {
            console.error('Error loading sample types:', error);
            this.showMessage('Failed to load sample types. Using default values.', 'error');
            
            // Fallback sample types for development/testing
            this.sample_types = [
                { id: '1', type_name: 'Blood' },
                { id: '2', type_name: 'Tissue' },
                { id: '3', type_name: 'Urine' },
                { id: '4', type_name: 'Saliva' }
            ];
            this.populateSampleTypeSelect();
        }
    }

    private populateSampleTypeSelect(): void 
    {
        // Clear existing options except the first one
        this.sample_type_select.innerHTML = '<option value="">Select a sample type...</option>';
        
        this.sample_types.forEach(sampleType => 
        {
            const option = document.createElement('option') as HTMLOptionElement;
            option.value = sampleType.id;
            option.textContent = sampleType.type_name;
            this.sample_type_select.appendChild(option);
        });
    }

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
