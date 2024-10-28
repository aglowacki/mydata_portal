async function connectToSSE() 
{
    const eventSource = new EventSource("http://localhost:3000/sse");

    eventSource.onmessage = (event) => 
    {
        console.log("Received SSE message:", event.data);
    };

    eventSource.onerror = (error) => 
    {
        console.error("SSE error:", error);
        eventSource.close();
    };

    console.log("Connected to SSE");
}

connectToSSE();