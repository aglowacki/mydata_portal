

export function show_toast(message: string, duration: number = 3000) 
{
    const toast = document.createElement("div");
    toast.classList.add("toast");
    toast.textContent = message;
    document.body.appendChild(toast);
  
    setTimeout(() => 
    {
      toast.remove();
    }, duration);
}