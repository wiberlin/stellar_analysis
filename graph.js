var should_merge = document.getElementById("merge_box").checked;

function make_chart_from_csv_url(canvas_id, csv_url) {
	const merged_title = "Merged by organization (nodes by the same organization count as 1)";
	const raw_title = "Raw nodes (each physical node counts as 1)";
	should_merge = document.getElementById("merge_box").checked;
	d3.csv(csv_url).then(function(csv_data) { make_chart_from_csv_data(canvas_id, csv_data); });
}

function make_chart_from_csv_data(canvas_id, csv_data) {

	Chart.defaults.global.defaultColor = 'rgba(0, 0, 0, 0)';

	var colors = {
		tt:        'rgba(70, 105, 90, 1)',
		mbs:       'rgba(35, 90, 130, 1)',
		mbs_light: 'rgba(35, 90, 130, 0.35)',
		mss:       'rgba(190, 85, 45, 1)',
		mss_light: 'rgba(190, 85, 45, 0.35)'
	};

	var chart_data = [];
	if (should_merge == false) {
		chart_data = {
			labels: csv_data.map(function(d) {return d.label}),
			datasets: [{
				data: csv_data.map(function(d) {return d.top_tier_size}),
				label: '|top tier|',
				borderColor: colors.tt,
				steppedLine: true,
				fill: -1
			}, {
				data: csv_data.map(function(d) {return d.mbs_mean}),
				label: 'mean(|minimal blocking sets|)',
				borderColor: colors.mbs,
				backgroundColor: colors.mbs_light,
				steppedLine: true,
				fill: -1
			}, {
				data: csv_data.map(function(d) {return d.mbs_min}),
				label: 'min(|minimal blocking sets|)',
				borderWidth: 0,
				backgroundColor: colors.mbs_light,
				steppedLine: true,
				fill: 1
			}, {
				data: csv_data.map(function(d) {return d.mbs_max}),
				label: 'max(|minimal blocking sets|)',
				borderWidth: 0,
				backgroundColor: colors.mbs_light,
				steppedLine: true,
				fill: 1
			}, {
				data: csv_data.map(function(d) {return d.mss_mean}),
				label: 'mean(|minimal splitting sets|)',
				borderColor: colors.mss,
				backgroundColor: colors.mss_light,
				steppedLine: true,
				fill: -1
			}, {
				data: csv_data.map(function(d) {return d.mss_min}),
				label: 'min(|minimal splitting sets|)',
				borderWidth: 0,
				backgroundColor: colors.mss_light,
				steppedLine: true,
				fill: 4
			}, {
				data: csv_data.map(function(d) {return d.mss_max}),
				label: 'max(|minimal splitting sets|)',
				borderWidth: 0,
				backgroundColor: colors.mss_light,
				steppedLine: true,
				fill: 4
			}]
		};
	} else {
		chart_data = {
			labels: csv_data.map(function(d) {return d.label}),
			datasets: [{
				data: csv_data.map(function(d) {return d.orgs_top_tier_size}),
				label: '|top tier|',
				borderColor: colors.tt,
				steppedLine: true,
				fill: -1
			}, {
				data: csv_data.map(function(d) {return d.orgs_mbs_mean}),
				label: 'mean(|minimal blocking sets|)',
				borderColor: colors.mbs,
				backgroundColor: colors.mbs_light,
				steppedLine: true,
				fill: -1
			}, {
				data: csv_data.map(function(d) {return d.orgs_mbs_min}),
				label: 'min(|minimal blocking sets|)',
				borderWidth: 0,
				backgroundColor: colors.mbs_light,
				steppedLine: true,
				fill: 1
			}, {
				data: csv_data.map(function(d) {return d.orgs_mbs_max}),
				label: 'max(|minimal blocking sets|)',
				borderWidth: 0,
				backgroundColor: colors.mbs_light,
				steppedLine: true,
				fill: 1
			}, {
				data: csv_data.map(function(d) {return d.orgs_mss_mean}),
				label: 'mean(|minimal splitting sets|)',
				borderColor: colors.mss,
				backgroundColor: colors.mss_light,
				steppedLine: true,
				fill: -1
			}, {
				data: csv_data.map(function(d) {return d.orgs_mss_min}),
				label: 'min(|minimal splitting sets|)',
				borderWidth: 0,
				backgroundColor: colors.mss_light,
				steppedLine: true,
				fill: 4
			}, {
				data: csv_data.map(function(d) {return d.orgs_mss_max}),
				label: 'max(|minimal splitting sets|)',
				borderWidth: 0,
				backgroundColor: colors.mss_light,
				steppedLine: true,
				fill: 4
			}]
		};

	}

	var chart_text;
	if (should_merge) {
		chart_text = "Merged by organization (nodes by the same organization count as 1)"
	} else {
		chart_text = "Raw nodes (each physical node counts as 1)";
	}

	var options = {
		responsive: true,
		maintainAspectRatio: false,
		title: {
			display: true,
			text: chart_text,
		},
		scales: {
			yAxes: [{
				ticks: {
					beginAtZero: true,
				}
			}]
		},
		elements: {
			point: {
				radius: 0,
				hitRadius: 5
			},
		},
		legend: {
			labels: {
				filter: function(legendItem, data) {
					return !legendItem.text.startsWith('min') && !legendItem.text.startsWith('max')
				}
			}
		}
	};

	var ctx = document.getElementById(canvas_id).getContext('2d');

	if (window.chart) window.chart.destroy();
	window.chart = new Chart(ctx, {
		type: 'line',
		data: chart_data,
		options: options
	});
	document.getElementById(canvas_id).onclick = function(evt) {
		var activePoint = chart.getElementAtEvent(event);

		// make sure click was on an actual point
		if (activePoint.length > 0) {
			var clickedDatasetIndex = activePoint[0]._datasetIndex;
			var clickedElementindex = activePoint[0]._index;
			var label = chart.data.labels[clickedElementindex];
			var value = chart.data.datasets[clickedDatasetIndex].data[clickedElementindex];     
			console.log("Clicked: " + label + " - " + value);
			call_stellarbeat_on_click(label);
		}
	};
}