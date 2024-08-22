
export function gen_anim()
{
    const main_div = document.createElement("div");

    const sample = document.createElement("div");
    sample.classList.add('sample');


    const x_particle0 = document.createElement("div");
    x_particle0.id = 'particle_0';
    x_particle0.classList.add('xray_particle');

    const x_particle1 = document.createElement("div");
    x_particle1.id = 'particle_1';
    x_particle1.classList.add('xray_particle');


    main_div.appendChild(x_particle0);
    main_div.appendChild(x_particle1);
    main_div.appendChild(sample);

    return main_div;
}