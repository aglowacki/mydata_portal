//import { gen_anim } from "./intro_anim.js";
import { get_cookie, has_cookie, set_cookie, delete_cookie } from "./cookies";
import { show_toast } from "./toast"


type User = {
    client_id: string;
    client_secret: string;
  };

interface LoginResponse {
    access_token: string;
    token_type: string;
}

// Mirrors the backend Claims struct returned by /api/user_info
export interface Claims {
    employee_id: string;
    mail: string;
    department: string;
    employee_type: string;
    sn: string;
    uac: string;
    exp: number;
}


// Decode the stored JWT and report whether it is missing, malformed, or past
// its `exp` claim. The backend rejects an expired token with a generic 400, so
// we inspect the token client-side to tell "expired" apart from other failures.
export function is_token_expired(): boolean
{
    const auth_cookie: string = get_cookie('access_token');
    if (!auth_cookie || auth_cookie === 'null')
    {
        return true;
    }

    const token = auth_cookie.replace(/^Bearer\s+/i, '');
    const parts = token.split('.');
    const encoded_payload = parts[1];
    if (parts.length !== 3 || !encoded_payload)
    {
        return true;
    }

    try
    {
        const payload = JSON.parse(atob(encoded_payload.replace(/-/g, '+').replace(/_/g, '/')));
        if (typeof payload.exp !== 'number')
        {
            return false;
        }
        // `exp` is seconds since epoch; Date.now() is milliseconds.
        return Date.now() >= payload.exp * 1000;
    }
    catch (error)
    {
        // A token we cannot parse is treated as unusable.
        return true;
    }
}

// Drop the stored credentials and send the user back to the login screen.
export function redirect_to_login(): void
{
    delete_cookie('access_token');
    window.location.href = '/';
}

// fetch() wrapper for authenticated API calls. If a request fails and the auth
// token has expired, clear it and bounce the user to the login screen instead
// of surfacing an opaque error.
export async function auth_fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>
{
    const response = await fetch(input, init);
    if (!response.ok && is_token_expired())
    {
        redirect_to_login();
    }
    return response;
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

// Fetch the current user's claims, reusing check_user()'s authenticated request.
export async function get_user_info(): Promise<Claims>
{
    const response = await check_user();
    return await response.json() as Claims;
}

export class LoginFormApp
{

    private main_div: HTMLDivElement;
    private login_form: HTMLFormElement;
    private username_input: HTMLInputElement;
    private password_input: HTMLInputElement;
    private submit_button: HTMLButtonElement;

    constructor() 
    {
        this.main_div = document.createElement("div") as HTMLDivElement;

        this.login_form = document.createElement("form") as HTMLFormElement;
        this.login_form.setAttribute('id', 'loginForm');
        this.login_form.classList.add("login_center");
        
        const logo_img = document.createElement('img');
        logo_img.src ='https://login.anl.gov/idp/images/argonne_logo.png';
        logo_img.width = 167;
        logo_img.height = 67;
        logo_img.classList.add("login_center");

        this.username_input = document.createElement("input") as HTMLInputElement;
        this.username_input.required = true;
        this.username_input.id = 'username';
        this.username_input.placeholder = 'Argonne Username';
        this.username_input.classList.add("login_center");

        this.password_input = document.createElement("input") as HTMLInputElement;
        this.password_input.type = 'password';
        this.password_input.required = true;
        this.password_input.id = 'password';
        this.password_input.placeholder = 'Password';
        this.password_input.classList.add("login_center");

        this.submit_button = document.createElement('button') as HTMLButtonElement;
        this.submit_button.textContent = 'Login';
        this.submit_button.type = 'submit';
        this.submit_button.classList.add("login_center");
        
        this.login_form.appendChild(logo_img);
        this.login_form.appendChild(document.createElement('br'));
        this.login_form.appendChild(this.username_input);
        this.login_form.appendChild(document.createElement('br'));
        this.login_form.appendChild(this.password_input);
        this.login_form.appendChild(document.createElement('br'));
        this.login_form.appendChild(this.submit_button);

        this.main_div.appendChild(this.login_form);

        this.setupEventListeners();
    }

    public gen_main_div(): HTMLDivElement
    {
        return this.main_div;
    }

    private setupEventListeners(): void
    {
        this.login_form.addEventListener('submit', (event) => this.handleLoginSubmit(event));
    }

    private handleLoginSubmit(event: Event)
    {
        event.preventDefault();
        const user: User = { client_id: this.username_input.value, client_secret: this.password_input.value };
        const resp = this.login_call(user);
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
    }

    private async login_call(user: User): Promise<LoginResponse> 
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

};
