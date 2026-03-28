<script lang="ts">
  import * as Card from '$lib/components/ui/card/index.js';
  import { Field, FieldLabel } from '$lib/components/ui/field/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Alert, AlertDescription } from '$lib/components/ui/alert/index.js';
  import { AuthService } from '$lib/services/auth';
  import { authStore } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  let username = $state('');
  let password = $state('');
  let isLoading = $state(false);
  let errorMessage = $state('');

  async function handleSubmit(event: Event) {
    event.preventDefault();
    isLoading = true;
    errorMessage = '';

    try {
      const result = await AuthService.login(username, password);

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
      if (error instanceof Error && error.message === '2FA_REQUIRED') {
        // Store credentials temporarily for OTP page
        sessionStorage.setItem('pending_2fa_username', username);
        sessionStorage.setItem('pending_2fa_password', password);
        goto('/otp');
        return;
      }
      errorMessage = error instanceof Error ? error.message : 'Login fehlgeschlagen';
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="flex min-h-screen items-center justify-center p-4">
  <Card.Root class="w-full max-w-md">
    <Card.Header class="text-center">
      <Card.Title class="text-2xl">Anmelden</Card.Title>
      <Card.Description>Melden Sie sich mit Ihrem Benutzernamen an</Card.Description>
    </Card.Header>
    <Card.Content>
      <form onsubmit={handleSubmit} class="space-y-4">
        {#if errorMessage}
          <Alert variant="destructive">
            <AlertDescription>{errorMessage}</AlertDescription>
          </Alert>
        {/if}

        <Field>
          <FieldLabel for="username">Benutzername</FieldLabel>
          <Input
            id="username"
            type="text"
            bind:value={username}
            placeholder="benutzername"
            disabled={isLoading}
            required
            autocomplete="username"
          />
        </Field>

        <Field>
          <FieldLabel for="password">Passwort</FieldLabel>
          <Input
            id="password"
            type="password"
            bind:value={password}
            placeholder="••••••••"
            disabled={isLoading}
            required
            autocomplete="current-password"
          />
        </Field>

        <Button type="submit" disabled={isLoading} class="w-full">
          {#if isLoading}
            <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"></div>
          {/if}
          Anmelden
        </Button>
      </form>
    </Card.Content>
  </Card.Root>
</div>
