import './assets/styles/main.scss'
import 'vue-final-modal/style.css'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import Notifications from '@kyvg/vue3-notification'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createVfm } from 'vue-final-modal'
import App from './App.vue'
import router from './router'
import { createMetaManager } from 'vue-meta'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { library } from '@fortawesome/fontawesome-svg-core'
import {
  faCalendar,
  faFileImage,
  faFileText,
  faGear,
  faUser,
  faBars,
  faX,
  faRightToBracket,
  faPenToSquare,
  faFloppyDisk,
  faArrowLeft,
  faHome,
  faArrowRight,
  faAnglesRight,
  faAnglesLeft,
  faEye,
  faEyeSlash,
  faUsers,
  faBoxOpen,
  faBoxesPacking,
  faToolbox,
  faUserPlus,
  faAngleDown,
  faCircleXmark,
  faCheckCircle
} from '@fortawesome/free-solid-svg-icons'

import { sessionStore } from './stores/session'
import { siteStore } from './stores/site'

const app = createApp(App)
const vfm = createVfm()
router.beforeEach((to) => {
  const store = sessionStore(pinia)
  if (to.meta.requiresAuth && store.session === undefined) {
    return {
      name: 'login'
    }
  } else if (to.meta.requiresIdentity === true && store.session === undefined) {
    return {
      name: 'login'
    }
  }
})

app.use(router)

/* add icons to the library */
library.add(faGear)
library.add(faUser)
library.add(faBars)
library.add(faFileText)
library.add(faFileImage)
library.add(faCalendar)
library.add(faRightToBracket)
library.add(faX)
library.add(faPenToSquare)
library.add(faFloppyDisk)
library.add(faArrowLeft)
library.add(faHome)
library.add(faArrowRight)
library.add(faAnglesRight)
library.add(faAnglesLeft)
library.add(faAngleDown)
library.add(faEye)
library.add(faUsers)
library.add(faBoxOpen)
library.add(faBoxesPacking)
library.add(faToolbox)
library.add(faUserPlus)
library.add(faCircleXmark)
library.add(faCheckCircle)
const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)
app.use(createMetaManager())
app.use(pinia)
app.component('font-awesome-icon', FontAwesomeIcon)
app.use(Notifications)

app.use(vfm)
app.mount('#app')
