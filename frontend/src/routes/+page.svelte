<script lang="ts">
  import { authStore } from '$lib/stores/auth';
  import { Button } from '$lib/components/ui/button/index.js';
  import { AuthService } from '$lib/services/auth';
  import { goto } from '$app/navigation';

  let authState = $derived($authStore);

  async function handleLogout() {
    if (authState.token) {
      await AuthService.logout(authState.token);
    }
    await fetch('/api/set-auth-cookie', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token: null }),
    });
    authStore.logout();
    goto('/signin');
  }
</script>

<div class="flex min-h-screen flex-col items-center justify-center gap-4">
  <h1 class="text-4xl font-bold">Welcome</h1>
  {#if authState.user}
    <p class="text-muted-foreground">Logged in as <strong>{authState.user.username}</strong></p>
    <div class="flex gap-2">
      <Button variant="outline" href="/settings">Settings</Button>
      <Button variant="destructive" onclick={handleLogout}>Logout</Button>
    </div>
  {/if}
</div>
