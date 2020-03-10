import Vue from 'vue'
import VueChartist from 'vue-chartist'

import App from './App.vue'

Vue.config.productionTip = false
Vue.use(VueChartist)

new Vue({
  render: h => h(App),
}).$mount('#app')
