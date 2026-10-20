import { createRouter, createWebHashHistory } from 'vue-router'
import MainLayout from '../components/layout/MainLayout.vue'
export const router = createRouter({ history: createWebHashHistory(), routes: [
{ path: '/', component: MainLayout, children: [
{ path: '', component: () => import('../views/DashboardView.vue') },
{ path: 'settings', component: () => import('../views/SettingsView.vue') }] },
{ path: '/overlay', component: () => import('../views/OverlayView.vue') },
{ path: '/:pathMatch(.*)*', redirect: '/' }] })
