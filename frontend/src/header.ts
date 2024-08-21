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
   // header_nav.appendChild(gen_signin_form());

    return header;
}

