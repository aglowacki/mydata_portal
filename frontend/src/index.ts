import { gen_login_form, check_user } from './auth.js';
import { gen_header } from "./header.js";
import { gen_left_sidebar } from "./left-sidebar.js";
import { gen_footer } from "./footer.js";

function main()
{   
    const app = document.getElementById("app");

    check_user().then(response => 
    {
                    console.log(response);
                    app?.classList.add("parent");
                    app?.appendChild(gen_header());
                    app?.appendChild(gen_left_sidebar());
                    //app?.appendChild(gen_center());
                    app?.appendChild(gen_footer());
    }).catch(error =>
    {
        //console.log(error);
        app?.appendChild(gen_login_form());
    }
    );    
}

window.onload = function() 
{   
    main();
};