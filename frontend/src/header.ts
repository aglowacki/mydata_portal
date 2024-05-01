function gen_beam_time_link()
{
    // if logged in and user has beam time
    const beam_a = document.createElement("a");
    beam_a.classList.add("nav");
    beam_a.href = "/beamline";
    beam_a.innerHTML = "Beamline";

    return beam_a;
}

function gen_datasets_link()
{
    // if logged in and user has beam time
    const datasets_a = document.createElement("a");
    datasets_a.classList.add("nav");
    datasets_a.href = "/datasets";
    datasets_a.innerHTML = "Datasets";

    return datasets_a;
}

function gen_signin_form()
{
    const form = document.createElement("form");
    form.classList.add("nav");
    form.classList.add("right");
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

    form.addEventListener('submit', (event) => {
        event.preventDefault();
        const username = username_input.value;
        const password = password_input.value;
        console.log(`Email: ${username}, Password: ${password}`);
        form.hidden = true;
        // Here you would typically send the email and password to your backend for authentication
      });

    return form;
}

function gen_user_icon()
{
    
/*
 <a href="#" class="desktop-item">Dropdown Menu</a>
        <input type="checkbox" id="showDrop">
        <label for="showDrop" class="mobile-item">Dropdown Menu</label>
        <ul class="drop-menu">
          <li><a href="#">Drop menu 1</a></li>
          <li><a href="#">Drop menu 2</a></li>
          <li><a href="#">Drop menu 3</a></li>
          <li><a href="#">Drop menu 4</a></li>
        </ul>
*/

}

export function gen_header()
{
    const header = document.createElement("header");
    header.classList.add("header");
    header.classList.add("nav");
    
    const header_nav = document.createElement("nav");
    header_nav.classList.add("nav");

    // always show home link in header
    const home_a = document.createElement("a");
    home_a.classList.add("nav");
    home_a.href = "/";
    home_a.innerHTML = "Home";

    header.appendChild(header_nav);
    header_nav.appendChild(home_a);
    header_nav.appendChild(gen_beam_time_link());
    header_nav.appendChild(gen_datasets_link());
    header_nav.appendChild(gen_signin_form());

    return header;
}

