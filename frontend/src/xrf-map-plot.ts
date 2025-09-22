import * as echarts from 'echarts/core';
import {
  TitleComponent,
  TitleComponentOption,
  TooltipComponent,
  TooltipComponentOption,
  GridComponent,
  GridComponentOption,
  VisualMapComponent,
  VisualMapComponentOption
} from 'echarts/components';
import { ScatterChart, ScatterSeriesOption } from 'echarts/charts';
import { UniversalTransition } from 'echarts/features';
import { HeatmapChart, HeatmapSeriesOption } from 'echarts/charts';
import { CanvasRenderer } from 'echarts/renderers';
import { get_cookie } from "./cookies.js";
import { show_toast } from "./toast.js"


function main()
{    

echarts.use([
  TitleComponent,
  TooltipComponent,
  GridComponent,
  VisualMapComponent,
  ScatterChart,
  CanvasRenderer,
  HeatmapChart,
  UniversalTransition
]);

type EChartsOption = echarts.ComposeOption<
  | TitleComponentOption
  | TooltipComponentOption
  | GridComponentOption
  | VisualMapComponentOption
  | ScatterSeriesOption
  | HeatmapSeriesOption
>;

var ROOT_PATH = '/';

var chartDom = document.getElementById('chart-container')!;
var myChart = echarts.init(chartDom, 'dark');
var option: EChartsOption;

option = {
  tooltip: {},
  grid: {
    right: 140,
    left: 40
  },
  xAxis: {
    type: 'category',
    data: []
  },
  yAxis: {
    type: 'category',
    data: []
  },
  visualMap: {
    type: 'piecewise',
    min: 0,
    max: 1,
    left: 'right',
    top: 'center',
    calculable: true,
    realtime: false,
    splitNumber: 8,
    inRange: {
      color: [
        '#313695',
        '#4575b4',
        '#74add1',
        '#abd9e9',
        '#e0f3f8',
        '#ffffbf',
        '#fee090',
        '#fdae61',
        '#f46d43',
        '#d73027',
        '#a50026'
      ]
    }
  },
  series: [
    {
      name: 'Gaussian',
      type: 'heatmap',
      data: [],
      emphasis: {
        itemStyle: {
          borderColor: '#333',
          borderWidth: 1
        }
      },
      progressive: 1000,
      animation: false
    }
  ]
};

option && myChart.setOption(option);

fetch('test_data.json')
  .then(response => response.json())
  .then(data => {
    myChart.setOption({
      xAxis: {
        data: data.xData
      },
      yAxis: {
        data: data.yData
      },
      series: [
        {
          data: data.data
        }
      ]
    });
  });
}


window.onload = function() 
{   
    main();
};

