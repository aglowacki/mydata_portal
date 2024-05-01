export function gen_header()
{
    const header = document.createElement("header");
    header.classList.add("header");
    
    const header_nav = document.createElement("nav");
    header_nav.classList.add("nav");

    const home_ul = document.createElement("ul");
    home_ul.innerHTML = "Home";
    home_ul.classList.add("nav");
    home_ul.classList.add("ul");

    header_nav.appendChild(home_ul);


    header.appendChild(header_nav);
    return header;
}

