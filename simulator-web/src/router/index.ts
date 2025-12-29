import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'SimulatorList',
      component: () => import('@/views/SimulatorList.vue'),
    },
    {
      path: '/simulator/:id',
      name: 'SimulatorDetail',
      component: () => import('@/views/SimulatorDetail.vue'),
      props: true,
    },
  ],
})

export default router
