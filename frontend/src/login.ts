
type User = {
    client_id: string;
    client_secret: string;
  };

interface LoginResponse {
    access_token: string;
    token_type: string; 
}

export function setCookie(name: string, val: string) 
{
    const date = new Date();
    const value = val;

    // Set it expire in 7 days
    date.setTime(date.getTime() + (7 * 24 * 60 * 60 * 1000));

    // Set it
    document.cookie = name+"="+value+"; expires="+date.toUTCString()+"; path=/";
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
          throw new Error(`HTTP error! status: ${response.status}`);
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
    
    const logo_img = document.createElement('img');
    logo_img.src ='https://login.anl.gov/idp/images/argonne_logo.png';
    logo_img.width = 167;
    logo_img.height = 67;

    const username_input = document.createElement("input");
    username_input.required = true;
    username_input.id = 'username';
    username_input.placeholder = 'Argonne Username';

    const password_input = document.createElement("input");
    password_input.type = 'password';
    password_input.required = true;
    password_input.id = 'password';
    password_input.placeholder = 'Password';

    const submit_button = document.createElement('button');
    submit_button.textContent = 'Login';
    submit_button.type = 'submit';
    
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
            setCookie('access_token', lres.token_type + " " +lres.access_token);
            window.location.reload();
        });
    });

    return form;
}

export function gen_login_form()
{
    const main_div = document.createElement("div");
    main_div.appendChild(gen_signin_form());

    return main_div;
}
