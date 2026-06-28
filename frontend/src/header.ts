import { get_user_info } from "./auth";
import { delete_cookie } from "./cookies";
import { get_effective_theme, toggle_theme } from "./theme";

function gen_link(txt: string, href: string)
{
    // if logged in and user has beam time
    const beam_a = document.createElement("a");
    beam_a.classList.add("nav");
    beam_a.href = href;
    beam_a.innerHTML = txt;

    return beam_a;
}

// Derive up to two initials from the user's name for the avatar label.
function initials_from(name: string): string
{
    const parts = name.trim().split(/\s+/).filter(Boolean);
    if (parts.length === 0)
    {
        return "?";
    }
    if (parts.length === 1)
    {
        return parts[0]!.charAt(0).toUpperCase();
    }
    return (parts[0]!.charAt(0) + parts[parts.length - 1]!.charAt(0)).toUpperCase();
}

function theme_label(): string
{
    // Label shows the theme the user would switch TO
    return get_effective_theme() === 'dark' ? "Switch to Light" : "Switch to Dark";
}

export function gen_user_icon()
{
    const menu = document.createElement("div");
    menu.classList.add("user-menu");

    const avatar = document.createElement("button");
    avatar.classList.add("user-avatar");
    avatar.type = "button";
    avatar.setAttribute("aria-label", "User menu");
    avatar.textContent = "\u{1F464}"; // generic person glyph until user info loads

    const dropdown = document.createElement("ul");
    dropdown.classList.add("user-dropdown");

    // View Profile
    const profile_li = document.createElement("li");
    const profile_a = document.createElement("a");
    profile_a.href = "/profile.html";
    profile_a.textContent = "View Profile";
    profile_li.appendChild(profile_a);

    // Theme toggle
    const theme_li = document.createElement("li");
    const theme_btn = document.createElement("button");
    theme_btn.type = "button";
    theme_btn.textContent = theme_label();
    theme_btn.addEventListener("click", (event) =>
    {
        event.stopPropagation();
        toggle_theme();
        theme_btn.textContent = theme_label();
    });
    theme_li.appendChild(theme_btn);

    // Log out
    const logout_li = document.createElement("li");
    const logout_btn = document.createElement("button");
    logout_btn.type = "button";
    logout_btn.textContent = "Log out";
    logout_btn.addEventListener("click", () =>
    {
        delete_cookie("access_token");
        window.location.href = "/";
    });
    logout_li.appendChild(logout_btn);

    dropdown.appendChild(profile_li);
    dropdown.appendChild(theme_li);
    dropdown.appendChild(logout_li);

    menu.appendChild(avatar);
    menu.appendChild(dropdown);

    // Toggle the dropdown when clicking the avatar
    avatar.addEventListener("click", (event) =>
    {
        event.stopPropagation();
        dropdown.classList.toggle("open");
    });

    // Close the dropdown when clicking anywhere outside the menu
    document.addEventListener("click", (event) =>
    {
        if (!menu.contains(event.target as Node))
        {
            dropdown.classList.remove("open");
        }
    });

    // Populate the avatar with the user's initials once their info loads
    get_user_info().then(claims =>
    {
        const name = claims.sn || claims.mail || "";
        if (name)
        {
            avatar.textContent = initials_from(name);
            avatar.title = name;
        }
    }).catch(() =>
    {
        // keep the generic glyph if user info is unavailable
    });

    return menu;
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

    header.appendChild(gen_user_icon());

    return header;
}
