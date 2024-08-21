
type User = {
    client_id: string;
    client_secret: string;
  };

export function setCookie(name: string, val: string) 
{
    const date = new Date();
    const value = val;

    // Set it expire in 7 days
    date.setTime(date.getTime() + (7 * 24 * 60 * 60 * 1000));

    // Set it
    document.cookie = name+"="+value+"; expires="+date.toUTCString()+"; path=/";
}

async function login_call(user: User): Promise<Response> 
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
            console.log(response);
            setCookie('Test_Auth', response.statusText);
            return response;
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
    
    const username_label = document.createElement("label");
    username_label.textContent = 'Username:';
    const username_input = document.createElement("input");
    //username_input.type = 'email';
    username_input.required = true;
    username_input.id = 'username';

    const password_label = document.createElement('label');
    password_label.textContent = 'Password:';
    const password_input = document.createElement("input");
    password_input.type = 'password';
    password_input.required = true;
    password_input.id = 'password';

    const submit_button = document.createElement('button');
    submit_button.textContent = 'Login';
    submit_button.type = 'submit';

    form.appendChild(username_label);
    form.appendChild(username_input);
    form.appendChild(password_label);
    form.appendChild(password_input);
    form.appendChild(submit_button);

    form.addEventListener('submit', (event) => 
    {
        event.preventDefault();
        const user: User = { client_id: username_input.value, client_secret: password_input.value };
        login_call(user);
    });

    return form;
}

export function gen_login_form()
{
    const main_div = document.createElement("div");
    main_div.appendChild(gen_signin_form());

    return main_div;
}
