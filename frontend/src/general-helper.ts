import { gen_login_form, check_user } from './auth';
import { gen_header } from "./header";
import { gen_left_sidebar } from "./left-sidebar";
import { gen_footer } from "./footer";

export function gen_index(id: string, center_element: HTMLElement)
{   
    const app = document.getElementById(id);

    check_user().then(response => 
    {
                    console.log(response);
                    app?.classList.add("parent");
                    app?.appendChild(gen_header());
                    app?.appendChild(gen_left_sidebar());
                    app?.appendChild(center_element!);
                    app?.appendChild(gen_footer());
    }).catch(error =>
    {
        //console.log(error);
        app?.appendChild(gen_login_form());
    }
    ); 
    return app;
}
