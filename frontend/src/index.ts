import { gen_login_form } from './login.js';
import { gen_header } from "./header.js";
import { gen_left_sidebar } from "./left-sidebar.js";
import { gen_footer } from "./footer.js";


function hasCookie(cookieName: string): boolean 
{
    return document.cookie.split(';').some(cookie => cookie.trim().startsWith(`${cookieName}=`));
}
//const getCookie = (cookieName: string) => new RegExp(`${cookieName}=([^;]+);`).exec(document.cookie)?.[1];
function getCookie(name: string): string 
{
    const match = document.cookie.match(new RegExp(name + '=([^;]+)'));
    return match ? decodeURIComponent(match[1]) : "null";
}

async function check_user(): Promise<Response> 
{
    const auth_cookie:string = getCookie('access_token');

    const headers = new Headers({
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'Authorization': auth_cookie,
      });
    
      const requestOptions: RequestInit = {
        method: 'GET',
        headers: headers,
      };

    try 
    {
        const response = await fetch('/api/user_info', requestOptions);
        if (!response.ok) 
        {
            if(response.status == 502)
            {
                throw new Error(`Backend Auth Serivce unreachable. ${response.status}`);
            }
            else if (response.status == 400)
            {
                throw new Error(`Missing credentials. ${response.status}`);
            }
            else
            { 
                throw new Error(`HTTP error! status: ${response.status}`);
            }
        }
        else
        {
          return response;
        }
    }
    catch (error) 
    {
        console.error('There was a problem with the fetch operation:', error);
        throw error;
    }

}

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