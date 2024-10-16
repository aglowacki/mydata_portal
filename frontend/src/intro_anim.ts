
export function gen_anim()
{
    const main_div = document.createElement("div");
    for (let i=0; i<6; i++)
    {
        const x_particle = document.createElement("div");
        x_particle.id = 'particle_'+i;
        x_particle.classList.add('xray_particle');
        main_div.appendChild(x_particle);
    }
    
    const sample = document.createElement("div");
    sample.classList.add('sample');
    main_div.appendChild(sample);

    const beam_pipe = document.createElement("div");
    beam_pipe.classList.add('silver-pipe');
    main_div.appendChild(beam_pipe);

    return main_div;
}