// Global clipboard handler for Tauri
document.addEventListener('keydown', async (event) => {
  // Handle paste with Cmd+V or Ctrl+V
  if ((event.metaKey || event.ctrlKey) && event.key === 'v') {
    const activeElement = document.activeElement;
    
    // Only handle if we're in an input or textarea
    if (activeElement && (activeElement.tagName === 'INPUT' || activeElement.tagName === 'TEXTAREA')) {
      event.preventDefault();
      event.stopPropagation();
      
      try {
        const text = await navigator.clipboard.readText();
        const input = activeElement;
        const start = input.selectionStart || 0;
        const end = input.selectionEnd || 0;
        const currentValue = input.value || '';
        
        // Insert text at cursor position
        const newValue = currentValue.slice(0, start) + text + currentValue.slice(end);
        input.value = newValue;
        
        // Dispatch input event for frameworks like Svelte
        input.dispatchEvent(new Event('input', { bubbles: true }));
        input.dispatchEvent(new Event('change', { bubbles: true }));
        
        // Set cursor position after pasted text
        setTimeout(() => {
          const newCursorPos = start + text.length;
          input.setSelectionRange(newCursorPos, newCursorPos);
        }, 0);
        
      } catch (err) {
        console.error('Paste failed:', err);
      }
    }
  }
  
  // Handle copy with Cmd+C or Ctrl+C
  if ((event.metaKey || event.ctrlKey) && event.key === 'c') {
    const activeElement = document.activeElement;
    
    if (activeElement && (activeElement.tagName === 'INPUT' || activeElement.tagName === 'TEXTAREA')) {
      const input = activeElement;
      const start = input.selectionStart || 0;
      const end = input.selectionEnd || 0;
      
      if (start !== end) {
        const selectedText = input.value.slice(start, end);
        try {
          await navigator.clipboard.writeText(selectedText);
          event.preventDefault();
          event.stopPropagation();
        } catch (err) {
          console.error('Copy failed:', err);
        }
      }
    }
  }
  
  // Handle select all with Cmd+A or Ctrl+A
  if ((event.metaKey || event.ctrlKey) && event.key === 'a') {
    const activeElement = document.activeElement;
    
    if (activeElement && (activeElement.tagName === 'INPUT' || activeElement.tagName === 'TEXTAREA')) {
      event.preventDefault();
      event.stopPropagation();
      activeElement.select();
    }
  }
}, true); // Use capture phase to catch events early