<script lang="ts">
  import * as Card from '$lib/components/ui/card/index.js';
  import * as Tabs from '$lib/components/ui/tabs/index.js';
  import { Field, FieldLabel, FieldDescription } from '$lib/components/ui/field/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Alert, AlertDescription } from '$lib/components/ui/alert/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { authStore } from '$lib/stores/auth';
  import { SettingsService } from '$lib/services/settings';
  import { AuthService } from '$lib/services/auth';
  import { onMount } from 'svelte';

  let email = $state('');
  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let isProfileLoading = $state(false);
  let profileMessage = $state('');

  let twoFactorEnabled = $state(false);
  let totpQrCode = $state('');
  let totpSecret = $state('');
  let verificationCode = $state('');
  let disableVerificationCode = $state('');
  let is2FALoading = $state(false);
  let twoFactorMessage = $state('');

  onMount(async () => {
    await new Promise((resolve) => setTimeout(resolve, 0));
    try {
      const profile = await SettingsService.getProfile();
      email = profile.email || '';
      twoFactorEnabled = profile.two_factor_enabled;
    } catch {
      // Silently fail — user will be redirected by layout if not authenticated
    }
  });

  async function handleEmailChange(event: Event) {
    event.preventDefault();
    isProfileLoading = true;
    profileMessage = '';
    try {
      await SettingsService.changeEmail(email);
      profileMessage = 'E-Mail erfolgreich aktualisiert';
    } catch (error) {
      profileMessage = error instanceof Error ? error.message : 'E-Mail-Aktualisierung fehlgeschlagen';
    } finally {
      isProfileLoading = false;
    }
  }

  async function handlePasswordChange(event: Event) {
    event.preventDefault();
    if (newPassword !== confirmPassword) {
      profileMessage = 'Passwörter stimmen nicht überein';
      return;
    }
    if (newPassword.length < 6) {
      profileMessage = 'Passwort muss mindestens 6 Zeichen lang sein';
      return;
    }
    isProfileLoading = true;
    profileMessage = '';
    try {
      const publicKey = await AuthService.getPublicKey();
      const encryptedOldPassword = await AuthService.encryptPassword(currentPassword, publicKey);
      const encryptedNewPassword = await AuthService.encryptPassword(newPassword, publicKey);
      await SettingsService.changePassword(encryptedOldPassword, encryptedNewPassword);
      profileMessage = 'Passwort erfolgreich geändert';
      currentPassword = '';
      newPassword = '';
      confirmPassword = '';
    } catch (error) {
      profileMessage = error instanceof Error ? error.message : 'Passwortänderung fehlgeschlagen';
    } finally {
      isProfileLoading = false;
    }
  }

  async function handleSetup2FA() {
    is2FALoading = true;
    twoFactorMessage = '';
    try {
      const response = await SettingsService.setup2FA();
      totpSecret = response.secret;
      totpQrCode = response.qr_code;
      twoFactorMessage = 'Scannen Sie den QR-Code mit Ihrer Authenticator App';
    } catch (error) {
      twoFactorMessage = error instanceof Error ? error.message : '2FA-Setup fehlgeschlagen';
    } finally {
      is2FALoading = false;
    }
  }

  async function handleEnable2FA(event: Event) {
    event.preventDefault();
    if (!verificationCode || verificationCode.length !== 6) {
      twoFactorMessage = 'Bitte geben Sie einen gültigen 6-stelligen Code ein';
      return;
    }
    is2FALoading = true;
    twoFactorMessage = '';
    try {
      await SettingsService.enable2FA(verificationCode);
      twoFactorEnabled = true;
      verificationCode = '';
      totpQrCode = '';
      totpSecret = '';
      twoFactorMessage = '2FA erfolgreich aktiviert';
    } catch (error) {
      twoFactorMessage = error instanceof Error ? error.message : '2FA-Aktivierung fehlgeschlagen';
    } finally {
      is2FALoading = false;
    }
  }

  async function handleDisable2FA(event: Event) {
    event.preventDefault();
    if (!disableVerificationCode || disableVerificationCode.length !== 6) {
      twoFactorMessage = 'Bitte geben Sie einen gültigen 6-stelligen Code ein';
      return;
    }
    is2FALoading = true;
    twoFactorMessage = '';
    try {
      await SettingsService.disable2FA(disableVerificationCode);
      twoFactorEnabled = false;
      disableVerificationCode = '';
      twoFactorMessage = '2FA erfolgreich deaktiviert';
    } catch (error) {
      twoFactorMessage = error instanceof Error ? error.message : '2FA-Deaktivierung fehlgeschlagen';
    } finally {
      is2FALoading = false;
    }
  }
</script>

<div class="flex-1 space-y-6 p-8 pt-6">
  <div class="space-y-1">
    <h2 class="text-3xl font-bold tracking-tight">Einstellungen</h2>
    <p class="text-muted-foreground">Verwalten Sie Ihr Profil und Ihre Sicherheitseinstellungen</p>
  </div>

  <Tabs.Root value="profile" class="space-y-6">
    <Tabs.List class="grid w-full max-w-2xl grid-cols-2">
      <Tabs.Trigger value="profile">Profil</Tabs.Trigger>
      <Tabs.Trigger value="security">Sicherheit</Tabs.Trigger>
    </Tabs.List>

    <Tabs.Content value="profile" class="space-y-6">
      <Card.Root>
        <Card.Header>
          <Card.Title>E-Mail ändern</Card.Title>
          <Card.Description>Aktualisieren Sie Ihre E-Mail-Adresse</Card.Description>
        </Card.Header>
        <Card.Content>
          <form onsubmit={handleEmailChange} class="space-y-4">
            {#if profileMessage && !profileMessage.includes('Passwort')}
              <Alert variant={profileMessage.includes('erfolgreich') ? 'default' : 'destructive'}>
                <AlertDescription>{profileMessage}</AlertDescription>
              </Alert>
            {/if}
            <Field>
              <FieldLabel for="email">E-Mail</FieldLabel>
              <Input
                id="email"
                type="email"
                bind:value={email}
                placeholder="ihre@email.de"
                disabled={isProfileLoading}
              />
            </Field>
            <Button type="submit" disabled={isProfileLoading}>
              {#if isProfileLoading}
                <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"></div>
              {/if}
              E-Mail aktualisieren
            </Button>
          </form>
        </Card.Content>
      </Card.Root>

      <Card.Root>
        <Card.Header>
          <Card.Title>Passwort ändern</Card.Title>
          <Card.Description>Ändern Sie Ihr Passwort für mehr Sicherheit</Card.Description>
        </Card.Header>
        <Card.Content>
          <form onsubmit={handlePasswordChange} class="space-y-4">
            {#if profileMessage && profileMessage.includes('Passwort')}
              <Alert variant={profileMessage.includes('erfolgreich') ? 'default' : 'destructive'}>
                <AlertDescription>{profileMessage}</AlertDescription>
              </Alert>
            {/if}
            <Field>
              <FieldLabel for="current-password">Aktuelles Passwort</FieldLabel>
              <Input
                id="current-password"
                type="password"
                bind:value={currentPassword}
                placeholder="••••••••"
                disabled={isProfileLoading}
              />
            </Field>
            <Field>
              <FieldLabel for="new-password">Neues Passwort</FieldLabel>
              <Input
                id="new-password"
                type="password"
                bind:value={newPassword}
                placeholder="••••••••"
                disabled={isProfileLoading}
              />
            </Field>
            <Field>
              <FieldLabel for="confirm-password">Passwort bestätigen</FieldLabel>
              <Input
                id="confirm-password"
                type="password"
                bind:value={confirmPassword}
                placeholder="••••••••"
                disabled={isProfileLoading}
              />
            </Field>
            <Button type="submit" disabled={isProfileLoading}>
              {#if isProfileLoading}
                <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"></div>
              {/if}
              Passwort ändern
            </Button>
          </form>
        </Card.Content>
      </Card.Root>
    </Tabs.Content>

    <Tabs.Content value="security" class="space-y-6">
      <Card.Root>
        <Card.Header>
          <Card.Title>Zwei-Faktor-Authentifizierung (2FA)</Card.Title>
          <Card.Description>
            Erhöhen Sie die Sicherheit Ihres Kontos mit TOTP
          </Card.Description>
        </Card.Header>
        <Card.Content class="space-y-6">
          {#if twoFactorMessage}
            <Alert variant={twoFactorMessage.includes('erfolgreich') ? 'default' : 'destructive'}>
              <AlertDescription>{twoFactorMessage}</AlertDescription>
            </Alert>
          {/if}

          <div class="space-y-4 rounded-lg border p-4">
            <div class="flex items-center gap-2">
              <p class="text-sm font-medium">Authenticator App (TOTP)</p>
              {#if twoFactorEnabled}
                <Badge variant="default">Aktiv</Badge>
              {:else}
                <Badge variant="secondary">Inaktiv</Badge>
              {/if}
            </div>

            {#if !twoFactorEnabled && !totpQrCode}
              <Button onclick={handleSetup2FA} disabled={is2FALoading} class="w-full">
                {#if is2FALoading}
                  <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"></div>
                {/if}
                2FA aktivieren
              </Button>
            {/if}

            {#if totpQrCode && !twoFactorEnabled}
              <div class="space-y-4">
                <div class="flex flex-col items-center gap-4 bg-white p-4 rounded-lg border">
                  <img
                    src="data:image/png;base64,{totpQrCode}"
                    alt="TOTP QR Code"
                    class="w-48 h-48"
                  />
                  <div class="text-center space-y-1">
                    <p class="text-sm font-medium text-foreground">Secret Key</p>
                    <code class="text-xs bg-muted px-2 py-1 rounded">{totpSecret}</code>
                  </div>
                </div>
                <form onsubmit={handleEnable2FA} class="space-y-4">
                  <Field>
                    <FieldLabel for="verification-code">Verifizierungscode</FieldLabel>
                    <Input
                      id="verification-code"
                      bind:value={verificationCode}
                      placeholder="123456"
                      maxlength={6}
                      disabled={is2FALoading}
                    />
                    <FieldDescription>6-stelliger Code aus Ihrer Authenticator App</FieldDescription>
                  </Field>
                  <Button type="submit" disabled={is2FALoading} class="w-full">
                    {#if is2FALoading}
                      <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"></div>
                    {/if}
                    Code verifizieren
                  </Button>
                </form>
              </div>
            {/if}

            {#if twoFactorEnabled}
              <form onsubmit={handleDisable2FA} class="space-y-4">
                <Field>
                  <FieldLabel for="disable-code">Code zum Deaktivieren</FieldLabel>
                  <Input
                    id="disable-code"
                    bind:value={disableVerificationCode}
                    placeholder="123456"
                    maxlength={6}
                    disabled={is2FALoading}
                  />
                </Field>
                <Button variant="destructive" type="submit" disabled={is2FALoading} class="w-full">
                  {#if is2FALoading}
                    <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"></div>
                  {/if}
                  2FA deaktivieren
                </Button>
              </form>
            {/if}
          </div>
        </Card.Content>
      </Card.Root>
    </Tabs.Content>
  </Tabs.Root>
</div>
