/* eslint-disable */
import 'normalize.css/normalize.css'
import './styles/main.scss'
import 'vue-final-modal/style.css'
import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import Notifications from '@kyvg/vue3-notification'
import { createMetaManager } from 'vue-meta'

import { createVfm } from 'vue-final-modal'
import { createPinia } from 'pinia'
/* import the fontawesome core */
import { library } from '@fortawesome/fontawesome-svg-core'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'

/* import font awesome icon component */
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

/* import specific icons */
import {
  faEye,
  faEyeSlash,
  faUserSecret,
  faFolder,
  faFileArrowDown
} from '@fortawesome/free-solid-svg-icons'
const vfm = createVfm()

/* add icons to the library */
library.add(faEyeSlash)
library.add(faEye)
library.add(faUserSecret)
library.add(faFolder)
library.add(faFileArrowDown)
const app = createApp(App)
app.use(Notifications)
app.use(router)
app.use(vfm)
app.use(createMetaManager())
const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)
app.use(createMetaManager())
app.component('font-awesome-icon', FontAwesomeIcon)

app.mount('#app')
