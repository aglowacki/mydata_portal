import { gen_index } from './general-helper';
import {get_sample_form} from "./samples"

window.onload = function() 
{   
    gen_index('app', get_sample_form());
};