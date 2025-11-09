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
