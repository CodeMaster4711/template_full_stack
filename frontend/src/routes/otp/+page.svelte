<script lang="ts">
  import * as Card from '$lib/components/ui/card/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Alert, AlertDescription } from '$lib/components/ui/alert/index.js';
  import { AuthService } from '$lib/services/auth';
  import { authStore } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  let code = $state('');
  let isLoading = $state(false);
  let errorMessage = $state('');
  let pendingUsername = $state('');
  let pendingPassword = $state('');

  onMount(() => {
    pendingUsername = sessionStorage.getItem('pending_2fa_username') ?? '';
    pendingPassword = sessionStorage.getItem('pending_2fa_password') ?? '';

    if (!pendingUsername || !pendingPassword) {
      goto('/signin');
    }
  });

  async function handleSubmit(event: Event) {
    event.preventDefault();

    if (code.length !== 6) {
      errorMessage = 'Bitte geben Sie einen 6-stelligen Code ein';
      return;
    }

    isLoading = true;
    errorMessage = '';

    try {
      const result = await AuthService.login(pendingUsername, pendingPassword, code);

      // Clear pending credentials
      sessionStorage.removeItem('pending_2fa_username');
      sessionStorage.removeItem('pending_2fa_password');

      // Set the auth cookie via server endpoint
      await fetch('/api/set-auth-cookie', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token: result.token }),
      });

      authStore.login(result.user, result.token);

      if (result.user.force_password_change) {
        goto('/change-password');
      } else {
        goto('/');
      }
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'Ungültiger Code';
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="flex min-h-screen items-center justify-center p-4">
  <Card.Root class="w-full max-w-md">
    <Card.Header class="text-center">
      <Card.Title class="text-2xl">Zwei-Faktor-Authentifizierung</Card.Title>
      <Card.Description>
        Geben Sie den 6-stelligen Code aus Ihrer Authenticator App ein
      </Card.Description>
    </Card.Header>
    <Card.Content>
      <form onsubmit={handleSubmit} class="space-y-4">
        {#if errorMessage}
          <Alert variant="destructive">
            <AlertDescription>{errorMessage}</AlertDescription>
          </Alert>
        {/if}

        <div class="flex justify-center">
          <Input
            type="text"
            bind:value={code}
            placeholder="000000"
            maxlength={6}
            disabled={isLoading}
            class="text-center text-2xl tracking-widest w-48"
            autocomplete="one-time-code"
            inputmode="numeric"
            pattern="[0-9]*"
          />
        </div>

        <Button type="submit" disabled={isLoading || code.length !== 6} class="w-full">
          {#if isLoading}
            <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"></div>
          {/if}
          Verifizieren
        </Button>

        <Button
          variant="ghost"
          class="w-full"
          onclick={() => goto('/signin')}
          disabled={isLoading}
        >
          Zurück zur Anmeldung
        </Button>
      </form>
    </Card.Content>
  </Card.Root>
</div>
