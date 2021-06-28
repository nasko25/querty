import { createWebHistory, createRouter } from 'vue-router'
import StartPage from '../components/StartPage.vue'

const routes = [
    { path: '/', component: StartPage }
]

const router = createRouter({
    history: createWebHistory(),
    routes,
})

export default router;
