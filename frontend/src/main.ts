/* eslint-disable */
import {createApp} from 'vue'
import App from './App.vue'
import router from './router'
import Notifications from '@kyvg/vue3-notification'
import {createMetaManager} from 'vue-meta'
import '@/styles/main.css'
import vfmPlugin from 'vue-final-modal'
import {createPinia} from 'pinia'

const app = createApp(App)
app.use(Notifications)
app.use(router)
app.use(vfmPlugin)
app.use(createMetaManager())
app.use(createPinia())

app.mount('#app')
