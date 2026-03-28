<script lang="ts">
  import '../app.css';
  import { authStore } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';

  let { data, children } = $props();

  const PUBLIC_ROUTES = ['/signin', '/otp', '/change-password'];

  onMount(() => {
    authStore.init(data.user ?? null, data.token ?? null);

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const apply = (dark: boolean) => document.documentElement.classList.toggle('dark', dark);
    apply(mediaQuery.matches);
    mediaQuery.addEventListener('change', (e) => apply(e.matches));
  });

  $effect(() => {
    const state = $authStore;
    if (state.isLoading) return;

    const currentPath = $page.url.pathname;
    const isPublicRoute = PUBLIC_ROUTES.some((route) => currentPath.startsWith(route));

    if (!state.isAuthenticated && !isPublicRoute) {
      goto('/signin');
    } else if (state.isAuthenticated && state.user?.force_password_change && currentPath !== '/change-password') {
      goto('/change-password');
    }
  });
</script>

<div class="h-full w-full">
  {@render children()}
</div>
