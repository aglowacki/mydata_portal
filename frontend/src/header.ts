function gen_link(txt: string, href: string)
{
    // if logged in and user has beam time
    const beam_a = document.createElement("a");
    beam_a.classList.add("nav");
    beam_a.href = href;
    beam_a.innerHTML = txt;

    return beam_a;
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

    header.appendChild(header_nav);
    header_nav.appendChild(gen_link("Home", "/"));
    header_nav.appendChild(gen_link("Proposals", "/proposals.html"));
    header_nav.appendChild(gen_link("Samples", "/samples.html"));
    //header_nav.appendChild(gen_signin_form());

    return header;
}

