import { gen_index } from './general-helper';
import {ProposalManagementApp} from "./proposals"

window.onload = function() 
{   
    let papp = new SampleManagementApp();
    gen_index('app', papp.gen_main_div());
};