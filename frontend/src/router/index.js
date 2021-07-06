import { createWebHistory, createRouter } from 'vue-router'
import StartPage from '../components/StartPage.vue'
import Results from '../components/Results.vue'

const routes = [
    { path: '/', component: StartPage },
    { path: '/results', component: Results }
]

const router = createRouter({
    history: createWebHistory(),
    routes,
})

export default router;
