<template>
<chartist
  class="circle"
  ratio="ct-chart"
  type="Pie"
  :data="chartData"
  :options="options"
  :event-handlers="eventHandlers"
/>
</template>

<script lang="ts">
import { Component, Vue, Prop} from "vue-property-decorator"

@Component
export default class GameCircle extends Vue {
  private chances: any[] = [{
    value: 100,
    meta: "#9146F7",
  }, {
    value: 100,
    meta: "#1116F7",
  }, {
    value: 100,
    meta: "#5196F7",
  }]

  private eventHandlers = [{
      event: "draw",
      fn: (data: any) => {
          if (data.type === "slice") {
            var pathLength = data.element._node.getTotalLength();
            data.element.attr({
              "stroke": data.meta,
              "stroke-width": "6px",
              'stroke-dasharray': pathLength + 'px ' + pathLength + 'px',
              'stroke-dashoffset': pathLength + 'px',
            });
            var animationDefinition: any = {
              'stroke-dashoffset': {
                id: 'anim' + data.index,
                dur: 1000,
                from: -pathLength + 'px',
                to:  '5px',
                easing: (this as any).$chartist.Svg.Easing.easeOutSine,
                fill: 'freeze'
              }
            };
            data.element.animate(animationDefinition, false);
          }
      },
  }]

  private get chartData() {
      return { series: this.chances }
  }

  private get options(): any {
      return {
          donut: true,
          strokeWidth: "6px",
          chartPadding: 30,
          showLabel: false,
      }
  }
}
</script>

<style scoped>
.circle {
  height: 100%;
  fill: none;
}
</style>
