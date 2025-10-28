interface SampleType {
    id: string;
    name: string;
}

interface Sample {
    sampleId: string;
    sampleName: string;
    sampleType: string;
    isGood: boolean;
    otherNotes: string;
}

class SampleManagementApp {
    private adminMode: boolean = false;
    private sampleTypes: SampleType[] = [];

    // DOM Elements
    private adminToggle: HTMLButtonElement;
    private adminPanel: HTMLDivElement;
    private sampleForm: HTMLFormElement;
    private sampleTypeSelect: HTMLSelectElement;
    private newSampleTypeInput: HTMLInputElement;
    private addSampleTypeButton: HTMLButtonElement;
    private messageDiv: HTMLDivElement;

    constructor() {
        this.initializeElements();
        this.setupEventListeners();
        this.loadSampleTypes();
    }

    private initializeElements(): void {
        this.adminToggle = document.getElementById('adminToggle') as HTMLButtonElement;
        this.adminPanel = document.getElementById('adminPanel') as HTMLDivElement;
        this.sampleForm = document.getElementById('sampleForm') as HTMLFormElement;
        this.sampleTypeSelect = document.getElementById('sampleType') as HTMLSelectElement;
        this.newSampleTypeInput = document.getElementById('newSampleType') as HTMLInputElement;
        this.addSampleTypeButton = document.getElementById('addSampleType') as HTMLButtonElement;
        this.messageDiv = document.getElementById('message') as HTMLDivElement;

        if (!this.adminToggle || !this.adminPanel || !this.sampleForm || 
            !this.sampleTypeSelect || !this.newSampleTypeInput || 
            !this.addSampleTypeButton || !this.messageDiv) {
            throw new Error('Required DOM elements not found');
        }
    }

    private setupEventListeners(): void {
        this.adminToggle.addEventListener('click', () => this.toggleAdminMode());
        this.addSampleTypeButton.addEventListener('click', () => this.addNewSampleType());
        this.sampleForm.addEventListener('submit', (e) => this.handleFormSubmit(e));
        
        // Allow Enter key to add sample type in admin mode
        this.newSampleTypeInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                this.addNewSampleType();
            }
        });
    }

    private toggleAdminMode(): void {
        this.adminMode = !this.adminMode;
        
        if (this.adminMode) {
            this.adminPanel.classList.remove('hidden');
            this.adminToggle.textContent = 'Exit Admin Mode';
            this.adminToggle.classList.add('active');
        } else {
            this.adminPanel.classList.add('hidden');
            this.adminToggle.textContent = 'Admin Mode';
            this.adminToggle.classList.remove('active');
        }
    }

    private async loadSampleTypes(): Promise<void> {
        try {
            const response = await fetch('/api/bio_sample_types');
            
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            
            this.sampleTypes = await response.json();
            this.populateSampleTypeSelect();
            this.showMessage('Sample types loaded successfully', 'success');
        } catch (error) {
            console.error('Error loading sample types:', error);
            this.showMessage('Failed to load sample types. Using default values.', 'error');
            
            // Fallback sample types for development/testing
            this.sampleTypes = [
                { id: '1', name: 'Blood' },
                { id: '2', name: 'Tissue' },
                { id: '3', name: 'Urine' },
                { id: '4', name: 'Saliva' }
            ];
            this.populateSampleTypeSelect();
        }
    }

    private populateSampleTypeSelect(): void {
        // Clear existing options except the first one
        this.sampleTypeSelect.innerHTML = '<option value="">Select a sample type...</option>';
        
        this.sampleTypes.forEach(sampleType => {
            const option = document.createElement('option');
            option.value = sampleType.id;
            option.textContent = sampleType.name;
            this.sampleTypeSelect.appendChild(option);
        });
    }

    private async addNewSampleType(): Promise<void> {
        const newTypeName = this.newSampleTypeInput.value.trim();
        
        if (!newTypeName) {
            this.showMessage('Please enter a sample type name', 'error');
            return;
        }

        // Check if sample type already exists
        if (this.sampleTypes.some(type => type.name.toLowerCase() === newTypeName.toLowerCase())) {
            this.showMessage('Sample type already exists', 'error');
            return;
        }

        try {
            const response = await fetch('/api/sample_types', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ name: newTypeName })
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const newSampleType: SampleType = await response.json();
            this.sampleTypes.push(newSampleType);
            this.populateSampleTypeSelect();
            this.newSampleTypeInput.value = '';
            this.showMessage(`Sample type "${newTypeName}" added successfully`, 'success');
        } catch (error) {
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

    private showMessage(text: string, type: 'success' | 'error'): void {
        this.messageDiv.textContent = text;
        this.messageDiv.className = `message ${type}`;
        this.messageDiv.classList.remove('hidden');
        
        // Auto-hide message after 5 seconds
        setTimeout(() => {
            this.messageDiv.classList.add('hidden');
        }, 5000);
    }
}

// Initialize the application when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new SampleManagementApp();
});
