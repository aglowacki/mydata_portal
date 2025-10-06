import { gen_index } from './general-helper';
import {gen_proposals_table} from "./proposals"

window.onload = function() 
{   
    gen_index('app', gen_proposals_table());
};