
const { invoke } = window.__TAURI__.tauri;
var mainWindow="main";
var configurationWindow="configuration";
const weekendsDays=[new Date(2024,0,1)]
function getWorkdaysArrayForCurrentMonth(data) {
    console.log(data);
    const currentDate = new Date();
    const currentMonth = currentDate.getMonth();
    const currentYear = currentDate.getFullYear();
    const firstDay = new Date(currentYear, currentMonth, 1,20);
    const lastDay = new Date(currentYear, currentMonth + 1, 0,20);

    const workdaysArray = [];
    let currentDay = firstDay;

    while (currentDay <= lastDay) {
        const dayOfWeek = currentDay.getDay();
        const formattedDate = getFormattedDate(currentDay);

        if (dayOfWeek !== 0 && dayOfWeek !== 6) {
            // Exclude Sunday (0) and Saturday (6)

            // Check if the date exists in worklogs.started
            const hasWorkLog = data.some(workLogDate => {
                var currentDayDate = currentDay.getDate();
                var existedFormatDate =  new Date(workLogDate.started).getDate();
                var isItWeekend  =weekendsDays.some(f=>f.getDate() === currentDayDate);
                return  isItWeekend|| currentDayDate=== existedFormatDate  ;
            });

            if (!hasWorkLog) {
                workdaysArray.push(formattedDate);
            }
        }

        currentDay.setDate(currentDay.getDate() + 1);
    }

    return workdaysArray;
}

function getFormattedDate(date){
    return date.toISOString().replace(/\.\d{3}Z$/, ".000+0000");
}
function parseTimeInput(inputStr) {
    // Regular expression to match time input in the format "Xd Xh Xm"
    const pattern = /((\d+)d\s*)?((\d+)h\s*)?((\d+)m\s*)?/;

    const match = inputStr.match(pattern);

    if (!match) {
        throw new Error("Invalid time input format. Please use the format Xd Xh Xm, e.g., 2d 3h 30m");
    }

    const [, days, , hours, , minutes] = match.map((matchGroup) => (matchGroup ? parseInt(matchGroup) : 0));

    const totalSeconds = (days || 0) * 24 * 60 * 60 + (hours || 0) * 60 * 60 + (minutes || 0) * 60;

    return totalSeconds;
}

function sendWorkLog(workLogText,worklogSec, formattedDate){
    invoke('worklog_task', {
        worklogText: workLogText,
        worklogSec: worklogSec,
        started: formattedDate
    })
        .then(f => {

        })
        .catch(error => {
            showError(error);
        });
}
window.addEventListener("DOMContentLoaded", () => {
    var worklogs =[]
    document.getElementById("trackMonth").addEventListener("click",()=>{

        var days = getWorkdaysArrayForCurrentMonth(worklogs);
        for(var i = 0; i<=days.length;i++){
            sendWorkLog("", 3600*8,days[i]);
        }
    })

    document.getElementById("trackDefaultButton").addEventListener("click",()=>{

        const currentDate = new Date();
        const formattedDate = getFormattedDate(currentDate)
        sendWorkLog("", 3600*8, formattedDate)
    })
    document.getElementById("trackButton").addEventListener("click",()=>{

        const currentDate = new Date();
        const formattedDate = getFormattedDate(currentDate)
        const text = document.getElementById("messageOfWorkLog").innerText;
        try {

            var duration = parseTimeInput(document.getElementById("timeWorklog").value)
            sendWorkLog(text, duration, formattedDate)
        }catch (exception){
            showError("put the same format as for jira");
            console.log(exception)
        }
        })
    document.getElementById('configuration').addEventListener('click', async function() {

        try {
            changeState(configurationWindow)
        }catch (  error ){
            showError(error)
        }
    });
    try {
        invoke('get_configuration').then(data=>{
            const jsonData = JSON.parse(data);

            document.getElementById("url").value = jsonData.url;
            document.getElementById("username").value = jsonData.username;
            document.getElementById("password").value = jsonData.password;
            if(jsonData.url!=""){
            invoke("get_task",{username: jsonData.username, url:jsonData.url, password: jsonData.password})
                .then(result=>{
                    var jsonData = JSON.parse(result);
                    var data = jsonData.Ok.fields;
                    worklogs=appendTrackedData(data);
                    appendSummaryData(data);
                }).catch(error=>{
                    showError(error);
            })}

        }).catch(error => {
            showError(error);
        });
    }catch (e) {
        showError(e)
    }
    document.getElementById("backButton").addEventListener("click",()=>{
        changeState(mainWindow)
    })
    document.getElementById("settingsSaveButton").addEventListener("click",()=>{

        invoke('set_configuration',
            {
                username: document.getElementById("username").value,
                password: document.getElementById("password").value,
                url: document.getElementById("url").value
            }).then(f=>{
            invoke("get_task",{username: jsonData.username, url:jsonData.url, password: jsonData.password})
                .then(result=>{
                    var jsonData = JSON.parse(result);
                    var data = jsonData.Ok.fields;
                    worklogs=appendTrackedData(data);
                    appendSummaryData(data);
                }).catch(error=>{
                showError(error);
            })
                changeState(mainWindow)
            }

        )

    })
});
function formatTime(seconds) {
    var hours = Math.floor(seconds / 3600);
    var minutes = Math.floor((seconds % 3600) / 60);
    var remainingSeconds = seconds % 60;

    return hours + "h:" + minutes + "m:" + remainingSeconds + "s";
}
function appendTrackedData(data){

    var trackedhours = document.getElementById("trackedHours");
    if(data.worklog.worklogs!=null){
        trackedhours.innerText =formatTime(data.worklog.worklogs.reduce((n, {timeSpentSeconds}) => n + timeSpentSeconds, 0));
        return data.worklog.worklogs;
    }else{
        trackedhours.innerText = 0;
        return [];
    }
}
function appendSummaryData(data){
     document.getElementById("summary").innerText =data.summary;;

}

function changeState(window){
    var configuration = document.getElementById("configuration-window");
    var main = document.getElementById("mainWindow");

    if(window===mainWindow){
        main.style.display="flex";
        configuration.style.display="none";
    }else if(window===configurationWindow){
        main.style.display="none";
        configuration.style.display="flex";
    }
}
function showError(errorMessage){
    document.getElementById("error-wrapper").textContent = errorMessage
}


