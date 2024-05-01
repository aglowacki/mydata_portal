import { gen_header } from "./header.js";
import { gen_left_sidebar } from "./left-sidebar.js";
import { gen_footer } from "./footer.js";

function main()
{
    const app = document.getElementById("app");
    app?.classList.add("parent");
    app?.appendChild(gen_header());
    app?.appendChild(gen_left_sidebar());
    app?.appendChild(gen_footer());

}

window.onload = function() 
{   
    main();
};