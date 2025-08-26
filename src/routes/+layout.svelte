<script>
  import '../app.css';
  import UpdateNotification from '$lib/components/UpdateNotification.svelte';
  import LogViewer from '$lib/components/LogViewer.svelte';
</script>

<UpdateNotification />
<LogViewer />
<slot />