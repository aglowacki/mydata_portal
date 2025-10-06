//import { gen_anim } from "./intro_anim.js";
import { get_cookie, has_cookie, set_cookie } from "./cookies";
import { show_toast } from "./toast"


type User = {
    client_id: string;
    client_secret: string;
  };

interface LoginResponse {
    access_token: string;
    token_type: string; 
}


export async function check_user(): Promise<Response> 
{
    const auth_cookie:string = get_cookie('access_token');

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

async function login_call(user: User): Promise<LoginResponse> 
{

    const headers = new Headers({
        'Content-Type': 'application/json',
        'Accept': 'application/json',
      });
    
      // Create the requestOptions object
      const requestOptions: RequestInit = {
        method: 'POST',
        headers: headers,
        body: JSON.stringify(user),
      };

    try 
    {
        const response = await fetch('/api/authorize', requestOptions);
        if (!response.ok) 
        {
            if( response.status == 502)
            {
                throw new Error(`Backend Auth Serivce unreachable. May be offline: ${response.status}`);
            }
            else if ( response.status == 401)
            {
                throw new Error(`Bad login credentials. Please Try again. ${response.status}`);
            }
            else
            {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
        }
        else
        {
            const data = await response.json();
            return data as LoginResponse;
        }
    }
    catch (error) 
    {
        console.error('There was a problem with the fetch operation:', error);
        throw error;
    }

}

function gen_signin_form()
{
    const form = document.createElement("form");
    //form.classList.add("nav");
    //form.classList.add("right");
    form.setAttribute('id', 'loginForm');
    form.classList.add("login_center");
    
    const logo_img = document.createElement('img');
    logo_img.src ='https://login.anl.gov/idp/images/argonne_logo.png';
    logo_img.width = 167;
    logo_img.height = 67;
    logo_img.classList.add("login_center");

    const username_input = document.createElement("input");
    username_input.required = true;
    username_input.id = 'username';
    username_input.placeholder = 'Argonne Username';
    username_input.classList.add("login_center");

    const password_input = document.createElement("input");
    password_input.type = 'password';
    password_input.required = true;
    password_input.id = 'password';
    password_input.placeholder = 'Password';
    password_input.classList.add("login_center");

    const submit_button = document.createElement('button');
    submit_button.textContent = 'Login';
    submit_button.type = 'submit';
    submit_button.classList.add("login_center");
    
    form.appendChild(logo_img);
    form.appendChild(document.createElement('br'));
    form.appendChild(username_input);
    form.appendChild(document.createElement('br'));
    form.appendChild(password_input);
    form.appendChild(document.createElement('br'));
    form.appendChild(submit_button);

    form.addEventListener('submit', (event) => 
    {
        event.preventDefault();
        const user: User = { client_id: username_input.value, client_secret: password_input.value };
        const resp = login_call(user);
        resp.then(lres =>
        {
            set_cookie('access_token', lres.token_type + " " +lres.access_token);
            window.location.reload();
        })
        .catch(error => 
        {
            show_toast(error.message);
            throw error;
        }
        );
    });

    return form;
}

export function gen_login_form()
{
    const main_div = document.createElement("div");
    main_div.appendChild(gen_signin_form());
    //main_div.appendChild(gen_anim());

    return main_div;
}
