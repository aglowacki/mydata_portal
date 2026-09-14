import { LoginFormApp, check_user } from './auth';
import type { Claims } from './auth';
import { gen_header } from "./header";
import { gen_left_sidebar } from "./left-sidebar";
import { gen_footer } from "./footer";
import { apply_theme } from "./theme";

export function gen_index(id: string, center_element: HTMLElement)
{
    apply_theme();
    const app = document.getElementById(id);

    check_user().then(async response =>
    {
                    // Read the caller's claims once and hand them to the header so
                    // the (Admin/Staff-only) Scheduler link renders on every page
                    // without a second, race-prone user_info lookup.
                    const claims = await response.json().catch(() => undefined) as Claims | undefined;
                    app?.classList.add("parent");
                    app?.appendChild(gen_header(claims));
                    app?.appendChild(gen_left_sidebar());
                    app?.appendChild(center_element!);
                    app?.appendChild(gen_footer());
    }).catch(error =>
    {
        //console.log(error);
        let loginapp = new LoginFormApp();
        app?.appendChild(loginapp.gen_main_div());
    }
    ); 
    return app;
}
