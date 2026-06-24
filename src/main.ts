import { createApp } from 'vue';
import { createPinia } from 'pinia';
import router from './router';
import App from './App.vue';
import './assets/theme.css';

const app = createApp(App);
const pinia = createPinia()

app.use(pinia);
app.use(router);

// Initialize theme after Pinia is installed (restore from localStorage)
import { useThemeStore } from './stores/theme';
const themeStore = useThemeStore()
themeStore.initTheme()

app.mount('#app');
