[x, Fs] = audioread('input2.wav'); 
g = 0.5; 
D = round(0.5 * Fs); 
y = x; 
for n = (D+1):length(x)
    y(n) = x(n) + g * x(n-D); 
end
audiowrite('output2_matlab_fir.wav', y, Fs); 



for n = (D+1):length(x)
    y(n) = x(n) + g * y(n-D); 
end

audiowrite('output2_matlab_iir.wav', y, Fs); 