import { ApiClient } from './api-client';

export interface Setup2FAResponse {
  secret: string;
  qr_code: string;
}

export class SettingsService {
  static async setup2FA(): Promise<Setup2FAResponse> {
    const response = await ApiClient.post('/2fa/setup');
    if (!response.ok) {
      throw new Error(`Failed to setup 2FA: ${response.statusText}`);
    }
    return response.json();
  }

  static async enable2FA(code: string): Promise<void> {
    const response = await ApiClient.post('/2fa/enable', { code });
    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Failed to enable 2FA');
    }
  }

  static async disable2FA(code: string): Promise<void> {
    const response = await ApiClient.post('/2fa/disable', { code });
    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Failed to disable 2FA');
    }
  }

  static async changePassword(oldPassword: string, newPassword: string): Promise<void> {
    const response = await ApiClient.post('/change-password', {
      old_password: oldPassword,
      new_password: newPassword,
    });
    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Failed to change password');
    }
  }

  static async changeEmail(newEmail: string): Promise<void> {
    const response = await ApiClient.post('/change-email', {
      new_email: newEmail,
    });
    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Failed to change email');
    }
  }

  static async getProfile(): Promise<{
    id: string;
    username: string;
    email?: string;
    two_factor_enabled: boolean;
    force_password_change: boolean;
  }> {
    const response = await ApiClient.get('/profile');
    if (!response.ok) {
      throw new Error(`Failed to get profile: ${response.statusText}`);
    }
    return response.json();
  }
}
