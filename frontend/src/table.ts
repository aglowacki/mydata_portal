document.addEventListener("DOMContentLoaded", () => 
{
    const table = document.createElement("table");
    table.className = "animated-table";
    document.body.appendChild(table);

    // Create table header
    const headerRow = table.insertRow();
    for (let i = 0; i < 5; i++) 
    {
        const th = document.createElement("th");
        th.innerText = `Header ${i + 1}`;
        headerRow.appendChild(th);
    }

    // Function to add a new row with animation
    function addRow() 
    {
        const row = table.insertRow();
        row.className = "new-row";
        for (let i = 0; i < 5; i++) 
        {
            const cell = row.insertCell();
            cell.innerText = `Data ${i + 1}`;
        }

        // Trigger animation by forcing reflow
        row.offsetWidth; 
        row.classList.add("visible");
    }

    // Add rows dynamically with delay
    let rowCount = 0;
    const interval = setInterval(() => 
    {
        if (rowCount < 10) 
        {
            addRow();
            rowCount++;
        }
        else
        {
            clearInterval(interval);
        }
    }, 1000);
});
