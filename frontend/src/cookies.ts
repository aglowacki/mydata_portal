
export function has_cookie(cookieName: string): boolean 
{
    return document.cookie.split(';').some(cookie => cookie.trim().startsWith(`${cookieName}=`));
}
//const getCookie = (cookieName: string) => new RegExp(`${cookieName}=([^;]+);`).exec(document.cookie)?.[1];
export function get_cookie(name: string): string 
{
    const match = document.cookie.match(new RegExp(name + '=([^;]+)'));
    return match ? decodeURIComponent(match[1]) : "null";
}

export function set_cookie(name: string, val: string) 
{
    const date = new Date();
    const value = val;

    // Set it expire in 7 days
    date.setTime(date.getTime() + (7 * 24 * 60 * 60 * 1000));

    // Set it
    document.cookie = name+"="+value+"; expires="+date.toUTCString()+"; path=/";
}
