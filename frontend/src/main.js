import { createApp } from 'vue'
import { createWebHistory, createRouter } from 'vue-router'
import App from './App.vue'
import StartPage from './components/StartPage.vue'

// TODO extract router to a separate page
const routes = [
    { path: '/', component: StartPage }
]

const router = createRouter({
    history: createWebHistory(),
    routes,
})

createApp(App)
    .use(router)
    .mount('#app')
